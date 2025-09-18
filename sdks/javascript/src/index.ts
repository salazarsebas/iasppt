export { DeAIClient } from './client';
export { DeAIWebSocketClient } from './websocket';
export * from './types';
export * from './errors';
export * from './utils';

// Re-export commonly used types for convenience
export type {
  TaskSubmissionRequest,
  TaskResponse,
  TaskResult,
  TaskStatus,
  NetworkStats,
  NodeInfo,
  ClientConfig,
} from './types';