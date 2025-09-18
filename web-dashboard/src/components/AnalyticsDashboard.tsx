import { useState } from 'react';
import { useQuery } from 'react-query';
import { formatDistanceToNow, format, subDays, subHours } from 'date-fns';
import { 
  ResponsiveContainer, 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  BarChart, 
  Bar,
  PieChart,
  Pie,
  Cell,
  AreaChart,
  Area
} from 'recharts';
import { apiClient } from '@/lib/api';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { 
  TrendingUpIcon,
  TrendingDownIcon,
  ActivityIcon,
  UsersIcon,
  ServerIcon,
  DollarSignIcon,
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  AlertTriangleIcon,
  RefreshCwIcon
} from 'lucide-react';

interface AnalyticsData {
  overview: {
    total_tasks: number;
    active_users: number;
    active_nodes: number;
    total_revenue: number;
    avg_processing_time: number;
    success_rate: number;
  };
  task_trends: Array<{
    date: string;
    total: number;
    completed: number;
    failed: number;
    processing_time_avg: number;
  }>;
  task_types: Array<{
    type: string;
    count: number;
    percentage: number;
  }>;
  node_performance: Array<{
    node_id: string;
    tasks_completed: number;
    avg_processing_time: number;
    success_rate: number;
    earnings: number;
  }>;
  user_activity: Array<{
    date: string;
    new_users: number;
    active_users: number;
    total_tasks: number;
  }>;
  revenue_data: Array<{
    date: string;
    revenue: number;
    tasks: number;
    avg_cost: number;
  }>;
}

type TimeRange = '24h' | '7d' | '30d' | '90d';

export function AnalyticsDashboard() {
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');
  const [refreshInterval, setRefreshInterval] = useState<number>(300000); // 5 minutes

  const { data: analytics, isLoading, error, refetch } = useQuery<AnalyticsData>(
    ['analytics', timeRange],
    () => apiClient.get(`/api/v1/admin/analytics?range=${timeRange}`),
    {
      refetchInterval: refreshInterval,
      staleTime: 60000, // Consider data stale after 1 minute
    }
  );

  const formatCurrency = (amount: number) => `${amount.toFixed(4)} NEAR`;
  
  const formatPercentage = (value: number) => `${value.toFixed(1)}%`;
  
  const formatDuration = (seconds: number) => {
    if (seconds < 60) return `${seconds.toFixed(1)}s`;
    if (seconds < 3600) return `${(seconds / 60).toFixed(1)}m`;
    return `${(seconds / 3600).toFixed(1)}h`;
  };

  const getTimeRangeLabel = (range: TimeRange) => {
    switch (range) {
      case '24h': return 'Last 24 Hours';
      case '7d': return 'Last 7 Days';
      case '30d': return 'Last 30 Days';
      case '90d': return 'Last 90 Days';
      default: return 'Unknown';
    }
  };

  const taskTypeColors = [
    '#8884d8', '#82ca9d', '#ffc658', '#ff7c7c', '#8dd1e1', '#d084d0'
  ];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error || !analytics) {
    return (
      <Card className="p-6">
        <div className="text-center text-red-600">
          <XCircleIcon className="w-8 h-8 mx-auto mb-2" />
          <p>Failed to load analytics data</p>
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

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Analytics Dashboard</h1>
          <p className="text-gray-600">DeAI Network Performance Metrics</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <Select value={timeRange} onValueChange={(value) => setTimeRange(value as TimeRange)}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="24h">Last 24 Hours</SelectItem>
              <SelectItem value="7d">Last 7 Days</SelectItem>
              <SelectItem value="30d">Last 30 Days</SelectItem>
              <SelectItem value="90d">Last 90 Days</SelectItem>
            </SelectContent>
          </Select>
          
          <Button 
            variant="outline" 
            size="sm" 
            onClick={() => refetch()}
          >
            <RefreshCwIcon className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Tasks</p>
                <p className="text-2xl font-bold">{analytics.overview.total_tasks.toLocaleString()}</p>
              </div>
              <ActivityIcon className="w-8 h-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Users</p>
                <p className="text-2xl font-bold">{analytics.overview.active_users.toLocaleString()}</p>
              </div>
              <UsersIcon className="w-8 h-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Nodes</p>
                <p className="text-2xl font-bold">{analytics.overview.active_nodes}</p>
              </div>
              <ServerIcon className="w-8 h-8 text-purple-600" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Revenue</p>
                <p className="text-2xl font-bold">{formatCurrency(analytics.overview.total_revenue)}</p>
              </div>
              <DollarSignIcon className="w-8 h-8 text-yellow-600" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Avg Processing Time</p>
                <p className="text-2xl font-bold">{formatDuration(analytics.overview.avg_processing_time)}</p>
              </div>
              <ClockIcon className="w-8 h-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Success Rate</p>
                <p className="text-2xl font-bold">{formatPercentage(analytics.overview.success_rate)}</p>
              </div>
              <CheckCircleIcon className="w-8 h-8 text-green-500" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Task Trends Chart */}
      <Card>
        <CardHeader>
          <CardTitle>Task Trends - {getTimeRangeLabel(timeRange)}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={analytics.task_trends}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="date" 
                  tickFormatter={(value) => format(new Date(value), 'MMM dd')}
                />
                <YAxis />
                <Tooltip 
                  labelFormatter={(value) => format(new Date(value), 'PPP')}
                  formatter={(value, name) => [
                    typeof value === 'number' ? value.toLocaleString() : value,
                    name === 'total' ? 'Total Tasks' : 
                    name === 'completed' ? 'Completed' : 'Failed'
                  ]}
                />
                <Area 
                  type="monotone" 
                  dataKey="total" 
                  stackId="1"
                  stroke="#8884d8" 
                  fill="#8884d8" 
                  fillOpacity={0.6}
                />
                <Area 
                  type="monotone" 
                  dataKey="completed" 
                  stackId="2"
                  stroke="#82ca9d" 
                  fill="#82ca9d" 
                  fillOpacity={0.6}
                />
                <Area 
                  type="monotone" 
                  dataKey="failed" 
                  stackId="3"
                  stroke="#ff7c7c" 
                  fill="#ff7c7c" 
                  fillOpacity={0.6}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Task Types Distribution */}
        <Card>
          <CardHeader>
            <CardTitle>Task Types Distribution</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={analytics.task_types}
                    cx="50%"
                    cy="50%"
                    outerRadius={80}
                    dataKey="count"
                    label={({ type, percentage }) => `${type} (${percentage.toFixed(1)}%)`}
                  >
                    {analytics.task_types.map((entry, index) => (
                      <Cell 
                        key={`cell-${index}`} 
                        fill={taskTypeColors[index % taskTypeColors.length]} 
                      />
                    ))}
                  </Pie>
                  <Tooltip 
                    formatter={(value, name) => [
                      typeof value === 'number' ? value.toLocaleString() : value,
                      'Tasks'
                    ]}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>

        {/* Revenue Trends */}
        <Card>
          <CardHeader>
            <CardTitle>Revenue Trends</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={analytics.revenue_data}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis 
                    dataKey="date" 
                    tickFormatter={(value) => format(new Date(value), 'MMM dd')}
                  />
                  <YAxis 
                    tickFormatter={(value) => formatCurrency(value)}
                  />
                  <Tooltip 
                    labelFormatter={(value) => format(new Date(value), 'PPP')}
                    formatter={(value, name) => [
                      name === 'revenue' ? formatCurrency(Number(value)) : 
                      name === 'tasks' ? Number(value).toLocaleString() :
                      formatCurrency(Number(value)),
                      name === 'revenue' ? 'Revenue' :
                      name === 'tasks' ? 'Tasks' : 'Avg Cost'
                    ]}
                  />
                  <Line 
                    type="monotone" 
                    dataKey="revenue" 
                    stroke="#8884d8" 
                    strokeWidth={2}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Node Performance Table */}
      <Card>
        <CardHeader>
          <CardTitle>Top Performing Nodes</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left p-3">Node ID</th>
                  <th className="text-left p-3">Tasks Completed</th>
                  <th className="text-left p-3">Avg Processing Time</th>
                  <th className="text-left p-3">Success Rate</th>
                  <th className="text-left p-3">Earnings</th>
                </tr>
              </thead>
              <tbody>
                {analytics.node_performance.slice(0, 10).map((node, index) => (
                  <tr key={node.node_id} className="border-b hover:bg-gray-50">
                    <td className="p-3">
                      <div className="flex items-center space-x-2">
                        <Badge variant="outline">#{index + 1}</Badge>
                        <span className="font-mono text-sm">{node.node_id.slice(0, 8)}...</span>
                      </div>
                    </td>
                    <td className="p-3">{node.tasks_completed.toLocaleString()}</td>
                    <td className="p-3">{formatDuration(node.avg_processing_time)}</td>
                    <td className="p-3">
                      <Badge 
                        variant={node.success_rate >= 95 ? "success" : 
                                node.success_rate >= 85 ? "default" : "destructive"}
                      >
                        {formatPercentage(node.success_rate)}
                      </Badge>
                    </td>
                    <td className="p-3">{formatCurrency(node.earnings)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* User Activity Chart */}
      <Card>
        <CardHeader>
          <CardTitle>User Activity Trends</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={analytics.user_activity}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="date" 
                  tickFormatter={(value) => format(new Date(value), 'MMM dd')}
                />
                <YAxis />
                <Tooltip 
                  labelFormatter={(value) => format(new Date(value), 'PPP')}
                  formatter={(value, name) => [
                    typeof value === 'number' ? value.toLocaleString() : value,
                    name === 'new_users' ? 'New Users' :
                    name === 'active_users' ? 'Active Users' : 'Total Tasks'
                  ]}
                />
                <Bar dataKey="new_users" fill="#8884d8" name="New Users" />
                <Bar dataKey="active_users" fill="#82ca9d" name="Active Users" />
                <Bar dataKey="total_tasks" fill="#ffc658" name="Total Tasks" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}