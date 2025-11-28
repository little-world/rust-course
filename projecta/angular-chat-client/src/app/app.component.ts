import { Component, OnInit, OnDestroy, ViewChild, ElementRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { WebSocketService } from './services/websocket.service';
import { ChatMessage, MessageType, ConnectionStatus } from './models/message.model';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements OnInit, OnDestroy {
  @ViewChild('messagesContainer') private messagesContainer!: ElementRef;

  title = 'WebSocket Chat Client';
  messages: ChatMessage[] = [];
  messageInput = '';
  usernameInput = '';
  connectionStatus = ConnectionStatus.Disconnected;

  // Expose enums to template
  MessageType = MessageType;
  ConnectionStatus = ConnectionStatus;

  private subscriptions = new Subscription();

  constructor(public wsService: WebSocketService) {}

  ngOnInit(): void {
    // Subscribe to messages
    this.subscriptions.add(
      this.wsService.messages$.subscribe(message => {
        this.messages.push(message);
        this.scrollToBottom();
      })
    );

    // Subscribe to connection status
    this.subscriptions.add(
      this.wsService.connectionStatus$.subscribe(status => {
        this.connectionStatus = status;
      })
    );

    // Auto-connect on startup
    this.wsService.connect();
  }

  ngOnDestroy(): void {
    this.subscriptions.unsubscribe();
    this.wsService.disconnect();
  }

  sendMessage(): void {
    const message = this.messageInput.trim();
    if (message && this.wsService.isConnected()) {
      this.wsService.send(message);
      this.messageInput = '';
    }
  }

  setUsername(): void {
    const username = this.usernameInput.trim();
    if (username) {
      this.wsService.setUsername(username);
      this.usernameInput = '';
    }
  }

  listUsers(): void {
    this.wsService.listUsers();
  }

  showStats(): void {
    this.wsService.showStats();
  }

  connect(): void {
    this.wsService.connect();
  }

  disconnect(): void {
    this.wsService.disconnect();
  }

  onKeyPress(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      this.sendMessage();
    }
  }

  getConnectionStatusClass(): string {
    return this.connectionStatus.toLowerCase();
  }

  getConnectionStatusText(): string {
    switch (this.connectionStatus) {
      case ConnectionStatus.Connected:
        return 'Connected';
      case ConnectionStatus.Connecting:
        return 'Connecting...';
      case ConnectionStatus.Disconnected:
        return 'Disconnected';
      case ConnectionStatus.Error:
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  }

  getMessageClass(type: MessageType): string {
    return `message-${type}`;
  }

  formatTimestamp(date: Date): string {
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    return `${hours}:${minutes}:${seconds}`;
  }

  private scrollToBottom(): void {
    setTimeout(() => {
      if (this.messagesContainer) {
        const element = this.messagesContainer.nativeElement;
        element.scrollTop = element.scrollHeight;
      }
    }, 100);
  }

  clearMessages(): void {
    this.messages = [];
  }
}
