import { createSignal, onCleanup } from 'solid-js';

type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface WebSocketMessage {
  type: string;
  payload: any;
}

class WebSocketManager {
  private socket: WebSocket | null = null;
  private url: string;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();
  private statusSignal = createSignal<WebSocketStatus>('disconnected');
  private reconnectTimeout: any = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = import.meta.env.VITE_WS_URL || 'localhost:3000';
    this.url = `${protocol}//${host}/api/ws`;
    this.connect();
  }

  private connect() {
    if (this.socket) {
      this.socket.close();
    }

    const [status, setStatus] = this.statusSignal;
    setStatus('connecting');

    const token = localStorage.getItem('auth_token');
    const wsUrl = token ? `${this.url}?token=${token}` : this.url;

    try {
      this.socket = new WebSocket(wsUrl);

      this.socket.onopen = () => {
        console.log('[WebSocket] Connected');
        setStatus('connected');
        this.reconnectAttempts = 0;
      };

      this.socket.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          const eventListeners = this.listeners.get(message.type);
          if (eventListeners) {
            eventListeners.forEach((callback) => callback(message.payload));
          }
        } catch (e) {
          console.error('[WebSocket] Failed to parse message:', e);
        }
      };

      this.socket.onclose = () => {
        console.log('[WebSocket] Disconnected');
        setStatus('disconnected');
        this.attemptReconnect();
      };

      this.socket.onerror = (error) => {
        console.error('[WebSocket] Error:', error);
        setStatus('error');
      };
    } catch (e) {
      console.error('[WebSocket] Connection failed:', e);
      setStatus('error');
      this.attemptReconnect();
    }
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
      console.log(`[WebSocket] Attempting reconnect in ${delay}ms...`);
      this.reconnectAttempts++;
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = setTimeout(() => this.connect(), delay);
    }
  }

  public on(event: string, callback: (data: any) => void) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
    return () => this.off(event, callback);
  }

  public off(event: string, callback: (data: any) => void) {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.delete(callback);
    }
  }

  public status() {
    return this.statusSignal[0]();
  }

  public send(type: string, payload: any) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ type, payload }));
    } else {
      console.warn('[WebSocket] Cannot send message: Socket not open');
    }
  }
}

// Singleton instance
const wsManager = new WebSocketManager();

export const useWebSocket = () => {
  return wsManager;
};

export default wsManager;
