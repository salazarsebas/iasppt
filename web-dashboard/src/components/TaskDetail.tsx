import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from 'react-query';
import { useRouter } from 'next/router';
import { formatDistanceToNow, format } from 'date-fns';
import { toast } from 'react-hot-toast';
import { apiClient } from '@/lib/api';
import { Task, TaskResult, TaskStatus } from '@/types/task';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { 
  ArrowLeftIcon,
  PlayIcon, 
  StopIcon, 
  CheckCircleIcon, 
  XCircleIcon,
  ClockIcon,
  RefreshCwIcon,
  DownloadIcon,
  CopyIcon,
  AlertTriangleIcon,
  InfoIcon
} from 'lucide-react';

interface TaskDetailProps {
  taskId: string;
}

export function TaskDetail({ taskId }: TaskDetailProps) {
  const router = useRouter();
  const queryClient = useQueryClient();
  const [autoRefresh, setAutoRefresh] = useState(true);

  const { data: task, isLoading: taskLoading, error: taskError, refetch: refetchTask } = useQuery(
    ['task', taskId],
    () => apiClient.get(`/api/v1/tasks/${taskId}`),
    {
      refetchInterval: autoRefresh && (!task || ['pending', 'processing'].includes(task.status)) ? 5000 : false,
    }
  );

  const { data: result, isLoading: resultLoading, error: resultError, refetch: refetchResult } = useQuery(
    ['task-result', taskId],
    () => apiClient.get(`/api/v1/tasks/${taskId}/result`),
    {
      enabled: task?.status === 'completed',
    }
  );

  const cancelTaskMutation = useMutation(
    () => apiClient.post(`/api/v1/tasks/${taskId}/cancel`),
    {
      onSuccess: () => {
        toast.success('Task cancelled successfully');
        refetchTask();
        queryClient.invalidateQueries(['tasks']);
      },
      onError: (error: any) => {
        toast.error(error.response?.data?.message || 'Failed to cancel task');
      },
    }
  );

  useEffect(() => {
    if (task && ['completed', 'failed', 'cancelled'].includes(task.status)) {
      setAutoRefresh(false);
    }
  }, [task?.status]);

  const getStatusIcon = (status: TaskStatus) => {
    switch (status) {
      case 'pending':
        return <ClockIcon className="w-5 h-5" />;
      case 'processing':
        return <PlayIcon className="w-5 h-5" />;
      case 'completed':
        return <CheckCircleIcon className="w-5 h-5" />;
      case 'failed':
        return <XCircleIcon className="w-5 h-5" />;
      case 'cancelled':
        return <StopIcon className="w-5 h-5" />;
      default:
        return <ClockIcon className="w-5 h-5" />;
    }
  };

  const getStatusColor = (status: TaskStatus) => {
    switch (status) {
      case 'pending':
        return 'secondary';
      case 'processing':
        return 'default';
      case 'completed':
        return 'success';
      case 'failed':
        return 'destructive';
      case 'cancelled':
        return 'outline';
      default:
        return 'secondary';
    }
  };

  const formatCost = (cost: number) => {
    return `${cost.toFixed(6)} NEAR`;
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    toast.success(`${label} copied to clipboard`);
  };

  const downloadResult = () => {
    if (!result) return;
    
    const dataStr = JSON.stringify(result, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `task-${taskId}-result.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    
    toast.success('Result downloaded successfully');
  };

  if (taskLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (taskError || !task) {
    return (
      <Card className="p-6">
        <div className="text-center text-red-600">
          <XCircleIcon className="w-8 h-8 mx-auto mb-2" />
          <p>Failed to load task details</p>
          <Button 
            variant="outline" 
            size="sm" 
            onClick={() => refetchTask()}
            className="mt-2"
          >
            <RefreshCwIcon className="w-4 h-4 mr-2" />
            Retry
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Button 
            variant="ghost" 
            size="sm"
            onClick={() => router.back()}
          >
            <ArrowLeftIcon className="w-4 h-4 mr-2" />
            Back
          </Button>
          
          <div>
            <h1 className="text-2xl font-bold">Task Details</h1>
            <p className="text-gray-600 font-mono">{taskId}</p>
          </div>
        </div>
        
        <div className="flex items-center space-x-3">
          {autoRefresh && (
            <Badge variant="outline" className="animate-pulse">
              Auto-refreshing
            </Badge>
          )}
          
          <Button 
            variant="outline" 
            size="sm" 
            onClick={() => refetchTask()}
          >
            <RefreshCwIcon className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          
          {(task.status === 'pending' || task.status === 'processing') && (
            <Button
              variant="destructive"
              size="sm"
              onClick={() => cancelTaskMutation.mutate()}
              disabled={cancelTaskMutation.isLoading}
            >
              <StopIcon className="w-4 h-4 mr-2" />
              Cancel Task
            </Button>
          )}
          
          {task.status === 'completed' && result && (
            <Button
              variant="outline"
              size="sm"
              onClick={downloadResult}
            >
              <DownloadIcon className="w-4 h-4 mr-2" />
              Download Result
            </Button>
          )}
        </div>
      </div>

      {/* Status Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-3">
            <Badge 
              variant={getStatusColor(task.status)}
              className="flex items-center space-x-2 text-base px-3 py-1"
            >
              {getStatusIcon(task.status)}
              <span className="capitalize">{task.status}</span>
            </Badge>
            
            <Badge variant="outline" className="text-base px-3 py-1">
              {task.task_type}
            </Badge>
            
            {task.priority > 5 && (
              <Badge variant="destructive" className="text-base px-3 py-1">
                High Priority ({task.priority}/10)
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div>
              <p className="text-sm font-medium text-gray-700 mb-1">Created</p>
              <p className="text-sm text-gray-900">
                {format(new Date(task.created_at), 'PPpp')}
              </p>
              <p className="text-xs text-gray-500">
                {formatDistanceToNow(new Date(task.created_at), { addSuffix: true })}
              </p>
            </div>
            
            {task.started_at && (
              <div>
                <p className="text-sm font-medium text-gray-700 mb-1">Started</p>
                <p className="text-sm text-gray-900">
                  {format(new Date(task.started_at), 'PPpp')}
                </p>
                <p className="text-xs text-gray-500">
                  {formatDistanceToNow(new Date(task.started_at), { addSuffix: true })}
                </p>
              </div>
            )}
            
            {task.completed_at && (
              <div>
                <p className="text-sm font-medium text-gray-700 mb-1">Completed</p>
                <p className="text-sm text-gray-900">
                  {format(new Date(task.completed_at), 'PPpp')}
                </p>
                <p className="text-xs text-gray-500">
                  Duration: {task.started_at 
                    ? formatDistanceToNow(new Date(task.started_at), { addSuffix: false })
                    : 'Unknown'
                  }
                </p>
              </div>
            )}
            
            <div>
              <p className="text-sm font-medium text-gray-700 mb-1">Cost</p>
              <p className="text-sm text-gray-900">
                {task.actual_cost ? formatCost(task.actual_cost) : 'Calculating...'}
              </p>
              {task.max_cost && (
                <p className="text-xs text-gray-500">
                  Max: {formatCost(task.max_cost)}
                </p>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Detailed Information */}
      <Tabs defaultValue="details" className="space-y-4">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="input">Input Data</TabsTrigger>
          {task.status === 'completed' && (
            <TabsTrigger value="result">Result</TabsTrigger>
          )}
          {(task.status === 'failed' || task.error_message) && (
            <TabsTrigger value="error">Error</TabsTrigger>
          )}
          <TabsTrigger value="logs">Logs</TabsTrigger>
        </TabsList>

        <TabsContent value="details">
          <Card>
            <CardHeader>
              <CardTitle>Task Configuration</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <p className="text-sm font-medium text-gray-700 mb-1">Model Name</p>
                  <div className="flex items-center space-x-2">
                    <p className="text-sm text-gray-900 font-mono">{task.model_name}</p>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard(task.model_name, 'Model name')}
                    >
                      <CopyIcon className="w-3 h-3" />
                    </Button>
                  </div>
                </div>
                
                <div>
                  <p className="text-sm font-medium text-gray-700 mb-1">Task Type</p>
                  <p className="text-sm text-gray-900">{task.task_type}</p>
                </div>
                
                <div>
                  <p className="text-sm font-medium text-gray-700 mb-1">Priority</p>
                  <p className="text-sm text-gray-900">{task.priority}/10</p>
                </div>
                
                {task.node_id && (
                  <div>
                    <p className="text-sm font-medium text-gray-700 mb-1">Assigned Node</p>
                    <div className="flex items-center space-x-2">
                      <p className="text-sm text-gray-900 font-mono">{task.node_id}</p>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => copyToClipboard(task.node_id!, 'Node ID')}
                      >
                        <CopyIcon className="w-3 h-3" />
                      </Button>
                    </div>
                  </div>
                )}
              </div>
              
              {task.parameters && Object.keys(task.parameters).length > 0 && (
                <div>
                  <p className="text-sm font-medium text-gray-700 mb-2">Parameters</p>
                  <div className="bg-gray-50 p-3 rounded-md">
                    <pre className="text-xs overflow-x-auto">
                      {JSON.stringify(task.parameters, null, 2)}
                    </pre>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="input">
          <Card>
            <CardHeader>
              <CardTitle>Input Data</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="bg-gray-50 p-4 rounded-md">
                <pre className="text-sm overflow-x-auto whitespace-pre-wrap">
                  {typeof task.input_data === 'string' 
                    ? task.input_data 
                    : JSON.stringify(task.input_data, null, 2)
                  }
                </pre>
              </div>
              <div className="mt-2 flex justify-end">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => copyToClipboard(
                    typeof task.input_data === 'string' 
                      ? task.input_data 
                      : JSON.stringify(task.input_data, null, 2),
                    'Input data'
                  )}
                >
                  <CopyIcon className="w-4 h-4 mr-2" />
                  Copy
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {task.status === 'completed' && (
          <TabsContent value="result">
            <Card>
              <CardHeader>
                <CardTitle>Task Result</CardTitle>
              </CardHeader>
              <CardContent>
                {resultLoading ? (
                  <div className="flex items-center justify-center p-8">
                    <LoadingSpinner size="md" />
                  </div>
                ) : resultError ? (
                  <div className="text-center text-red-600 p-4">
                    <XCircleIcon className="w-6 h-6 mx-auto mb-2" />
                    <p>Failed to load result</p>
                    <Button 
                      variant="outline" 
                      size="sm" 
                      onClick={() => refetchResult()}
                      className="mt-2"
                    >
                      <RefreshCwIcon className="w-4 h-4 mr-2" />
                      Retry
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-4">
                    <div className="bg-gray-50 p-4 rounded-md max-h-96 overflow-y-auto">
                      <pre className="text-sm whitespace-pre-wrap">
                        {JSON.stringify(result, null, 2)}
                      </pre>
                    </div>
                    <div className="flex justify-end space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => copyToClipboard(JSON.stringify(result, null, 2), 'Result')}
                      >
                        <CopyIcon className="w-4 h-4 mr-2" />
                        Copy
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={downloadResult}
                      >
                        <DownloadIcon className="w-4 h-4 mr-2" />
                        Download
                      </Button>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>
        )}

        {(task.status === 'failed' || task.error_message) && (
          <TabsContent value="error">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-red-600">
                  <AlertTriangleIcon className="w-5 h-5" />
                  <span>Error Information</span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                {task.error_message ? (
                  <div className="bg-red-50 border border-red-200 p-4 rounded-md">
                    <pre className="text-sm text-red-900 whitespace-pre-wrap">
                      {task.error_message}
                    </pre>
                  </div>
                ) : (
                  <div className="text-center text-gray-500 p-4">
                    <InfoIcon className="w-6 h-6 mx-auto mb-2" />
                    <p>No error information available</p>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>
        )}

        <TabsContent value="logs">
          <Card>
            <CardHeader>
              <CardTitle>Processing Logs</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-center text-gray-500 p-8">
                <InfoIcon className="w-8 h-8 mx-auto mb-2" />
                <p>Detailed processing logs will be available here</p>
                <p className="text-sm">Feature coming soon...</p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}