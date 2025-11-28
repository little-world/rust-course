import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable, Subject } from 'rxjs';
import { ChatMessage, MessageType, ConnectionStatus } from '../models/message.model';

@Injectable({
  providedIn: 'root'
})
export class WebSocketService {
  private socket: WebSocket | null = null;
  private readonly serverUrl = 'ws://127.0.0.1:8080';

  private messagesSubject = new Subject<ChatMessage>();
  private connectionStatusSubject = new BehaviorSubject<ConnectionStatus>(
    ConnectionStatus.Disconnected
  );

  public messages$: Observable<ChatMessage> = this.messagesSubject.asObservable();
  public connectionStatus$: Observable<ConnectionStatus> =
    this.connectionStatusSubject.asObservable();

  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectTimeout: any;

  connect(): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      console.log('Already connected');
      return;
    }

    this.connectionStatusSubject.next(ConnectionStatus.Connecting);
    this.addSystemMessage('Connecting to server...');

    try {
      this.socket = new WebSocket(this.serverUrl);

      this.socket.onopen = () => {
        console.log('WebSocket connected');
        this.connectionStatusSubject.next(ConnectionStatus.Connected);
        this.addSystemMessage('Connected to server');
        this.reconnectAttempts = 0;
      };

      this.socket.onmessage = (event) => {
        this.handleMessage(event.data);
      };

      this.socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.connectionStatusSubject.next(ConnectionStatus.Error);
        this.addSystemMessage('WebSocket error occurred');
      };

      this.socket.onclose = () => {
        console.log('WebSocket disconnected');
        this.connectionStatusSubject.next(ConnectionStatus.Disconnected);
        this.addSystemMessage('Disconnected from server');
        this.socket = null;
        this.attemptReconnect();
      };
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      this.connectionStatusSubject.next(ConnectionStatus.Error);
      this.addSystemMessage('Failed to connect to server');
    }
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.addSystemMessage('Maximum reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    this.addSystemMessage(`Reconnecting in ${delay / 1000} seconds... (Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

    this.reconnectTimeout = setTimeout(() => {
      this.connect();
    }, delay);
  }

  disconnect(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    this.reconnectAttempts = this.maxReconnectAttempts; // Prevent auto-reconnect

    if (this.socket) {
      if (this.socket.readyState === WebSocket.OPEN) {
        this.send('/quit');
      }
      this.socket.close();
      this.socket = null;
    }

    this.connectionStatusSubject.next(ConnectionStatus.Disconnected);
  }

  send(message: string): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(message);

      // Echo sent messages locally (if not a command)
      if (!message.startsWith('/')) {
        this.messagesSubject.next({
          content: `You: ${message}`,
          type: MessageType.Sent,
          timestamp: new Date()
        });
      }
    } else {
      this.addSystemMessage('Not connected to server');
    }
  }

  setUsername(username: string): void {
    this.send(`/name ${username}`);
  }

  listUsers(): void {
    this.send('/list');
  }

  showStats(): void {
    this.send('/stats');
  }

  whisper(username: string, message: string): void {
    this.send(`/whisper ${username} ${message}`);
  }

  private handleMessage(data: string): void {
    let messageType = MessageType.Received;

    // Determine message type based on content
    if (data.includes('Whisper')) {
      messageType = MessageType.Whisper;
    } else if (data.startsWith('Error:') || data.includes('not found') || data.includes('already taken')) {
      messageType = MessageType.Error;
    } else if (
      data.includes('Username set') ||
      data.includes('Users online') ||
      data.includes('Statistics') ||
      data.includes('Goodbye')
    ) {
      messageType = MessageType.System;
    }

    this.messagesSubject.next({
      content: data,
      type: messageType,
      timestamp: new Date()
    });
  }

  private addSystemMessage(message: string): void {
    this.messagesSubject.next({
      content: message,
      type: MessageType.System,
      timestamp: new Date()
    });
  }

  isConnected(): boolean {
    return this.socket?.readyState === WebSocket.OPEN;
  }

  ngOnDestroy(): void {
    this.disconnect();
  }
}
