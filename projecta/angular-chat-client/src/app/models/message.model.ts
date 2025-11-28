export interface ChatMessage {
  content: string;
  type: MessageType;
  timestamp: Date;
}

export enum MessageType {
  System = 'system',
  Received = 'received',
  Sent = 'sent',
  Whisper = 'whisper',
  Error = 'error'
}

export enum ConnectionStatus {
  Connected = 'connected',
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Error = 'error'
}
