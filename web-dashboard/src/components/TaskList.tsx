import { useState } from 'react';
import { useQuery } from 'react-query';
import { useRouter } from 'next/router';
import { formatDistanceToNow, format } from 'date-fns';
import { apiClient } from '@/lib/api';
import { Task, TaskStatus } from '@/types/task';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { 
  PlayIcon, 
  PauseIcon, 
  StopIcon, 
  CheckCircleIcon, 
  XCircleIcon,
  ClockIcon,
  SearchIcon,
  RefreshCwIcon,
  DownloadIcon,
  EyeIcon
} from 'lucide-react';

interface TaskListProps {
  showAllTasks?: boolean;
  userId?: string;
}

export function TaskList({ showAllTasks = false, userId }: TaskListProps) {
  const router = useRouter();
  const [statusFilter, setStatusFilter] = useState<TaskStatus | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 10;

  const { data: tasksData, isLoading, error, refetch } = useQuery(
    ['tasks', statusFilter, searchQuery, currentPage, showAllTasks, userId],
    async () => {
      const params = new URLSearchParams({
        page: currentPage.toString(),
        limit: pageSize.toString(),
      });
      
      if (statusFilter !== 'all') {
        params.append('status', statusFilter);
      }
      
      if (searchQuery) {
        params.append('search', searchQuery);
      }
      
      if (showAllTasks) {
        return apiClient.get(`/api/v1/admin/tasks?${params.toString()}`);
      } else if (userId) {
        return apiClient.get(`/api/v1/admin/users/${userId}/tasks?${params.toString()}`);
      } else {
        return apiClient.get(`/api/v1/tasks?${params.toString()}`);
      }
    },
    {
      refetchInterval: 5000, // Refresh every 5 seconds
      keepPreviousData: true,
    }
  );

  const handleTaskClick = (taskId: string) => {
    router.push(`/tasks/${taskId}`);
  };

  const handleCancelTask = async (taskId: string, event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      await apiClient.post(`/api/v1/tasks/${taskId}/cancel`);
      refetch();
    } catch (error) {
      console.error('Failed to cancel task:', error);
    }
  };

  const getStatusIcon = (status: TaskStatus) => {
    switch (status) {
      case 'pending':
        return <ClockIcon className="w-4 h-4" />;
      case 'processing':
        return <PlayIcon className="w-4 h-4" />;
      case 'completed':
        return <CheckCircleIcon className="w-4 h-4" />;
      case 'failed':
        return <XCircleIcon className="w-4 h-4" />;
      case 'cancelled':
        return <StopIcon className="w-4 h-4" />;
      default:
        return <ClockIcon className="w-4 h-4" />;
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

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <Card className="p-6">
        <div className="text-center text-red-600">
          <XCircleIcon className="w-8 h-8 mx-auto mb-2" />
          <p>Failed to load tasks</p>
          <Button 
            variant="outline" 
            size="sm" 
            onClick={() => refetch()}
            className="mt-2"
          >
            <RefreshCwIcon className="w-4 h-4 mr-2" />
            Retry
          </Button>
        </div>
      </Card>
    );
  }

  const tasks = tasksData?.data || [];
  const pagination = tasksData?.pagination;

  return (
    <div className="space-y-4">
      {/* Filters and Search */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Tasks</span>
            <Button 
              variant="outline" 
              size="sm" 
              onClick={() => refetch()}
            >
              <RefreshCwIcon className="w-4 h-4 mr-2" />
              Refresh
            </Button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                <Input
                  placeholder="Search tasks by ID, model, or input..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <div className="w-full sm:w-48">
              <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value as TaskStatus | 'all')}>
                <SelectTrigger>
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="pending">Pending</SelectItem>
                  <SelectItem value="processing">Processing</SelectItem>
                  <SelectItem value="completed">Completed</SelectItem>
                  <SelectItem value="failed">Failed</SelectItem>
                  <SelectItem value="cancelled">Cancelled</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Task List */}
      {tasks.length === 0 ? (
        <Card className="p-8">
          <div className="text-center text-gray-500">
            <ClockIcon className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <h3 className="text-lg font-medium mb-2">No tasks found</h3>
            <p>
              {searchQuery || statusFilter !== 'all'
                ? 'Try adjusting your filters'
                : 'Submit your first AI task to get started'}
            </p>
          </div>
        </Card>
      ) : (
        <div className="space-y-3">
          {tasks.map((task: Task) => (
            <Card 
              key={task.id} 
              className="cursor-pointer hover:shadow-md transition-shadow"
              onClick={() => handleTaskClick(task.id)}
            >
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-3 mb-2">
                      <Badge 
                        variant={getStatusColor(task.status)}
                        className="flex items-center space-x-1"
                      >
                        {getStatusIcon(task.status)}
                        <span className="capitalize">{task.status}</span>
                      </Badge>
                      
                      <Badge variant="outline">
                        {task.task_type}
                      </Badge>
                      
                      {task.priority > 5 && (
                        <Badge variant="destructive">
                          High Priority
                        </Badge>
                      )}
                    </div>
                    
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                      <div>
                        <p className="text-sm font-medium text-gray-700">Task ID</p>
                        <p className="text-sm text-gray-600 font-mono">{task.id.slice(0, 8)}...</p>
                      </div>
                      
                      <div>
                        <p className="text-sm font-medium text-gray-700">Model</p>
                        <p className="text-sm text-gray-600 truncate">{task.model_name}</p>
                      </div>
                      
                      <div>
                        <p className="text-sm font-medium text-gray-700">Created</p>
                        <p className="text-sm text-gray-600">
                          {formatDistanceToNow(new Date(task.created_at), { addSuffix: true })}
                        </p>
                      </div>
                      
                      <div>
                        <p className="text-sm font-medium text-gray-700">Cost</p>
                        <p className="text-sm text-gray-600">
                          {task.actual_cost ? formatCost(task.actual_cost) : 'Calculating...'}
                        </p>
                      </div>
                    </div>
                    
                    {showAllTasks && (
                      <div className="mt-2">
                        <p className="text-sm font-medium text-gray-700">User</p>
                        <p className="text-sm text-gray-600">{task.user_id}</p>
                      </div>
                    )}
                    
                    <div className="mt-3">
                      <p className="text-sm font-medium text-gray-700">Input Preview</p>
                      <p className="text-sm text-gray-600 truncate max-w-2xl">
                        {typeof task.input_data === 'string' 
                          ? task.input_data 
                          : JSON.stringify(task.input_data)
                        }
                      </p>
                    </div>
                    
                    {task.node_id && (
                      <div className="mt-2">
                        <p className="text-sm font-medium text-gray-700">Assigned Node</p>
                        <p className="text-sm text-gray-600 font-mono">{task.node_id}</p>
                      </div>
                    )}
                  </div>
                  
                  <div className="flex flex-col space-y-2 ml-4">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleTaskClick(task.id);
                      }}
                    >
                      <EyeIcon className="w-4 h-4 mr-2" />
                      View
                    </Button>
                    
                    {task.status === 'completed' && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          // TODO: Implement result download
                        }}
                      >
                        <DownloadIcon className="w-4 h-4 mr-2" />
                        Download
                      </Button>
                    )}
                    
                    {(task.status === 'pending' || task.status === 'processing') && (
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={(e) => handleCancelTask(task.id, e)}
                      >
                        <StopIcon className="w-4 h-4 mr-2" />
                        Cancel
                      </Button>
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Pagination */}
      {pagination && pagination.total_pages > 1 && (
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-600">
                Showing {((pagination.current_page - 1) * pagination.per_page) + 1} to{' '}
                {Math.min(pagination.current_page * pagination.per_page, pagination.total_items)} of{' '}
                {pagination.total_items} tasks
              </div>
              
              <div className="flex space-x-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={pagination.current_page <= 1}
                  onClick={() => setCurrentPage(pagination.current_page - 1)}
                >
                  Previous
                </Button>
                
                <div className="flex items-center space-x-1">
                  {[...Array(Math.min(5, pagination.total_pages))].map((_, index) => {
                    const page = index + 1;
                    const isActive = page === pagination.current_page;
                    
                    return (
                      <Button
                        key={page}
                        variant={isActive ? "default" : "outline"}
                        size="sm"
                        onClick={() => setCurrentPage(page)}
                      >
                        {page}
                      </Button>
                    );
                  })}
                </div>
                
                <Button
                  variant="outline"
                  size="sm"
                  disabled={pagination.current_page >= pagination.total_pages}
                  onClick={() => setCurrentPage(pagination.current_page + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}