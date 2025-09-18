import axios, { AxiosInstance, AxiosError } from 'axios';
import { EventEmitter } from 'eventemitter3';
import { v4 as uuidv4 } from 'uuid';
import {
  ClientConfig,
  TaskSubmissionRequest,
  TaskResponse,
  TaskResult,
  TaskStatus,
  NetworkStats,
  NodeInfo,
  UserProfile,
  ApiKey,
  PaginatedResponse,
  AuthResponse,
} from './types';
import { DeAIError, TaskError, AuthError, NetworkError } from './errors';
import { validateTaskRequest, formatNearAmount } from './utils';

/**
 * Main DeAI SDK client for interacting with the DeAI computation network
 */
export class DeAIClient extends EventEmitter {
  private api: AxiosInstance;
  private config: Required<ClientConfig>;
  private accessToken?: string;

  constructor(config: ClientConfig) {
    super();

    // Apply defaults
    this.config = {
      apiUrl: config.apiUrl || 'https://api.deai.org',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      retryDelay: config.retryDelay || 1000,
      ...config,
    };

    // Create axios instance
    this.api = axios.create({
      baseURL: this.config.apiUrl,
      timeout: this.config.timeout,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': `DeAI-SDK-JS/0.1.0`,
      },
    });

    // Setup request interceptor for auth
    this.api.interceptors.request.use((config) => {
      if (this.accessToken) {
        config.headers.Authorization = `Bearer ${this.accessToken}`;
      }
      return config;
    });

    // Setup response interceptor for error handling
    this.api.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        const deaiError = this.handleApiError(error);
        this.emit('error', deaiError);
        throw deaiError;
      }
    );
  }

  // Authentication Methods

  /**
   * Login with username and password
   */
  async login(username: string, password: string): Promise<UserProfile> {
    try {
      const response = await this.api.post<AuthResponse>('/api/v1/auth/login', {
        username,
        password,
      });

      this.accessToken = response.data.access_token;
      this.emit('authenticated', response.data.user);
      
      return response.data.user;
    } catch (error) {
      throw new AuthError('Login failed', { cause: error });
    }
  }

  /**
   * Login with Near wallet signature
   */
  async loginWithNear(
    accountId: string,
    publicKey: string,
    signature: string,
    message: string
  ): Promise<UserProfile> {
    try {
      const response = await this.api.post<AuthResponse>('/api/v1/auth/near-login', {
        account_id: accountId,
        public_key: publicKey,
        signature,
        message,
      });

      this.accessToken = response.data.access_token;
      this.emit('authenticated', response.data.user);
      
      return response.data.user;
    } catch (error) {
      throw new AuthError('Near wallet login failed', { cause: error });
    }
  }

  /**
   * Set API key for authentication
   */
  setApiKey(apiKey: string): void {
    this.accessToken = apiKey;
    this.emit('authenticated');
  }

  /**
   * Logout and clear authentication
   */
  logout(): void {
    this.accessToken = undefined;
    this.emit('logout');
  }

  // Task Management Methods

  /**
   * Submit a new AI task
   */
  async submitTask(request: TaskSubmissionRequest): Promise<TaskResponse> {
    validateTaskRequest(request);

    try {
      const response = await this.api.post<TaskResponse>('/api/v1/tasks', request);
      
      const task = response.data;
      this.emit('taskSubmitted', task);
      
      // Auto-poll for updates if requested
      if (this.config.autoPoll) {
        this.pollTaskUpdates(task.id);
      }
      
      return task;
    } catch (error) {
      throw new TaskError('Failed to submit task', { cause: error });
    }
  }

  /**
   * Get task information
   */
  async getTask(taskId: string): Promise<TaskResponse> {
    try {
      const response = await this.api.get<TaskResponse>(`/api/v1/tasks/${taskId}`);
      return response.data;
    } catch (error) {
      throw new TaskError(`Failed to get task ${taskId}`, { cause: error });
    }
  }

  /**
   * Get task result
   */
  async getTaskResult(taskId: string): Promise<TaskResult> {
    try {
      const response = await this.api.get<TaskResult>(`/api/v1/tasks/${taskId}/result`);
      return response.data;
    } catch (error) {
      throw new TaskError(`Failed to get task result ${taskId}`, { cause: error });
    }
  }

  /**
   * List user's tasks with pagination
   */
  async listTasks(options?: {
    page?: number;
    limit?: number;
    status?: TaskStatus;
  }): Promise<PaginatedResponse<TaskResponse>> {
    try {
      const params = new URLSearchParams();
      if (options?.page) params.append('page', options.page.toString());
      if (options?.limit) params.append('limit', options.limit.toString());
      if (options?.status) params.append('status', options.status);

      const response = await this.api.get<PaginatedResponse<TaskResponse>>(
        `/api/v1/tasks?${params.toString()}`
      );
      return response.data;
    } catch (error) {
      throw new TaskError('Failed to list tasks', { cause: error });
    }
  }

  /**
   * Cancel a task
   */
  async cancelTask(taskId: string): Promise<TaskResponse> {
    try {
      const response = await this.api.post<TaskResponse>(`/api/v1/tasks/${taskId}/cancel`);
      const task = response.data;
      this.emit('taskCancelled', task);
      return task;
    } catch (error) {
      throw new TaskError(`Failed to cancel task ${taskId}`, { cause: error });
    }
  }

  /**
   * Wait for task completion
   */
  async waitForCompletion(
    taskId: string,
    options?: {
      timeout?: number;
      pollInterval?: number;
    }
  ): Promise<TaskResult> {
    const timeout = options?.timeout || 300000; // 5 minutes default
    const pollInterval = options?.pollInterval || 5000; // 5 seconds default
    const startTime = Date.now();

    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const task = await this.getTask(taskId);
          
          if (task.status === 'completed') {
            const result = await this.getTaskResult(taskId);
            this.emit('taskCompleted', { task, result });
            resolve(result);
            return;
          }
          
          if (task.status === 'failed' || task.status === 'cancelled') {
            const error = new TaskError(`Task ${taskId} ${task.status}`);
            this.emit('taskFailed', { task, error });
            reject(error);
            return;
          }
          
          // Check timeout
          if (Date.now() - startTime > timeout) {
            reject(new TaskError(`Task ${taskId} timed out`));
            return;
          }
          
          // Continue polling
          setTimeout(poll, pollInterval);
        } catch (error) {
          reject(error);
        }
      };

      poll();
    });
  }

  // Network Information Methods

  /**
   * Get network statistics
   */
  async getNetworkStats(): Promise<NetworkStats> {
    try {
      const response = await this.api.get<NetworkStats>('/api/v1/network/stats');
      return response.data;
    } catch (error) {
      throw new NetworkError('Failed to get network stats', { cause: error });
    }
  }

  /**
   * List active nodes
   */
  async listNodes(): Promise<NodeInfo[]> {
    try {
      const response = await this.api.get<NodeInfo[]>('/api/v1/nodes');
      return response.data;
    } catch (error) {
      throw new NetworkError('Failed to list nodes', { cause: error });
    }
  }

  /**
   * Get specific node information
   */
  async getNode(nodeId: string): Promise<NodeInfo> {
    try {
      const response = await this.api.get<NodeInfo>(`/api/v1/nodes/${nodeId}`);
      return response.data;
    } catch (error) {
      throw new NetworkError(`Failed to get node ${nodeId}`, { cause: error });
    }
  }

  // User Management Methods

  /**
   * Get user profile
   */
  async getProfile(): Promise<UserProfile> {
    try {
      const response = await this.api.get<UserProfile>('/api/v1/user/profile');
      return response.data;
    } catch (error) {
      throw new AuthError('Failed to get profile', { cause: error });
    }
  }

  /**
   * Create API key
   */
  async createApiKey(name: string, expiresInDays?: number): Promise<ApiKey> {
    try {
      const response = await this.api.post<ApiKey>('/api/v1/user/api-keys', {
        name,
        expires_in_days: expiresInDays,
      });
      return response.data;
    } catch (error) {
      throw new AuthError('Failed to create API key', { cause: error });
    }
  }

  /**
   * List API keys
   */
  async listApiKeys(): Promise<ApiKey[]> {
    try {
      const response = await this.api.get<ApiKey[]>('/api/v1/user/api-keys');
      return response.data;
    } catch (error) {
      throw new AuthError('Failed to list API keys', { cause: error });
    }
  }

  /**
   * Revoke API key
   */
  async revokeApiKey(keyId: string): Promise<void> {
    try {
      await this.api.post(`/api/v1/user/api-keys/${keyId}/revoke`);
    } catch (error) {
      throw new AuthError(`Failed to revoke API key ${keyId}`, { cause: error });
    }
  }

  // Utility Methods

  /**
   * Check if client is authenticated
   */
  isAuthenticated(): boolean {
    return !!this.accessToken;
  }

  /**
   * Get current API configuration
   */
  getConfig(): Required<ClientConfig> {
    return { ...this.config };
  }

  /**
   * Update client configuration
   */
  updateConfig(config: Partial<ClientConfig>): void {
    this.config = { ...this.config, ...config };
    
    // Update axios instance if URL changed
    if (config.apiUrl) {
      this.api.defaults.baseURL = config.apiUrl;
    }
    
    if (config.timeout) {
      this.api.defaults.timeout = config.timeout;
    }
  }

  // Private Methods

  private async pollTaskUpdates(taskId: string): void {
    const poll = async () => {
      try {
        const task = await this.getTask(taskId);
        this.emit('taskUpdated', task);
        
        if (task.status === 'completed' || task.status === 'failed' || task.status === 'cancelled') {
          return; // Stop polling
        }
        
        setTimeout(poll, 5000); // Poll every 5 seconds
      } catch (error) {
        this.emit('error', error);
      }
    };

    setTimeout(poll, 1000); // Start polling after 1 second
  }

  private handleApiError(error: AxiosError): DeAIError {
    const status = error.response?.status;
    const data = error.response?.data as any;
    const message = data?.message || error.message;

    switch (status) {
      case 400:
        return new DeAIError('Bad Request', { cause: error, details: data });
      case 401:
        return new AuthError('Unauthorized', { cause: error });
      case 403:
        return new AuthError('Forbidden', { cause: error });
      case 404:
        return new DeAIError('Not Found', { cause: error });
      case 429:
        return new DeAIError('Rate Limited', { cause: error });
      case 500:
        return new NetworkError('Internal Server Error', { cause: error });
      default:
        return new NetworkError(message, { cause: error });
    }
  }
}