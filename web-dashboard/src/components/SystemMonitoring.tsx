import { useState, useEffect } from 'react';
import { useQuery } from 'react-query';
import { formatDistanceToNow, format } from 'date-fns';
import { 
  ResponsiveContainer, 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip,
  AreaChart,
  Area
} from 'recharts';
import { apiClient } from '@/lib/api';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Progress } from '@/components/ui/Progress';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { 
  ServerIcon,
  CpuIcon,
  HardDriveIcon,
  WifiIcon,
  AlertTriangleIcon,
  CheckCircleIcon,
  XCircleIcon,
  ActivityIcon,
  RefreshCwIcon,
  ZapIcon,
  DatabaseIcon
} from 'lucide-react';

interface SystemMetrics {
  timestamp: string;
  api_gateway: {
    status: 'healthy' | 'degraded' | 'down';
    response_time: number;
    requests_per_minute: number;
    error_rate: number;
    active_connections: number;
  };
  database: {
    status: 'healthy' | 'degraded' | 'down';
    connections: number;
    query_time_avg: number;
    storage_used: number;
    storage_total: number;
  };
  redis: {
    status: 'healthy' | 'degraded' | 'down';
    memory_used: number;
    memory_total: number;
    keys_count: number;
    hit_rate: number;
  };
  blockchain: {
    status: 'healthy' | 'degraded' | 'down';
    latest_block: number;
    sync_status: 'synced' | 'syncing' | 'behind';
    transaction_pool: number;
  };
  nodes: {
    total: number;
    active: number;
    processing: number;
    offline: number;
    avg_load: number;
  };
  resource_usage: Array<{
    timestamp: string;
    cpu_usage: number;
    memory_usage: number;
    disk_usage: number;
    network_io: number;
  }>;
  alerts: Array<{
    id: string;
    level: 'info' | 'warning' | 'error' | 'critical';
    message: string;
    timestamp: string;
    resolved: boolean;
  }>;
}

export function SystemMonitoring() {
  const [autoRefresh, setAutoRefresh] = useState(true);

  const { data: metrics, isLoading, error, refetch } = useQuery<SystemMetrics>(
    ['system-metrics'],
    () => apiClient.get('/api/v1/admin/system/metrics'),
    {
      refetchInterval: autoRefresh ? 10000 : false, // 10 seconds
      staleTime: 5000,
    }
  );

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'success';
      case 'degraded': return 'warning';
      case 'down': return 'destructive';
      default: return 'secondary';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy': return <CheckCircleIcon className="w-4 h-4" />;
      case 'degraded': return <AlertTriangleIcon className="w-4 h-4" />;
      case 'down': return <XCircleIcon className="w-4 h-4" />;
      default: return <ActivityIcon className="w-4 h-4" />;
    }
  };

  const getAlertIcon = (level: string) => {
    switch (level) {
      case 'critical': return <XCircleIcon className="w-4 h-4 text-red-600" />;
      case 'error': return <XCircleIcon className="w-4 h-4 text-red-500" />;
      case 'warning': return <AlertTriangleIcon className="w-4 h-4 text-yellow-500" />;
      case 'info': return <CheckCircleIcon className="w-4 h-4 text-blue-500" />;
      default: return <ActivityIcon className="w-4 h-4" />;
    }
  };

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatPercentage = (value: number) => `${value.toFixed(1)}%`;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error || !metrics) {
    return (
      <Card className="p-6">
        <div className="text-center text-red-600">
          <XCircleIcon className="w-8 h-8 mx-auto mb-2" />
          <p>Failed to load system metrics</p>
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

  const activeAlerts = metrics.alerts.filter(alert => !alert.resolved);
  const criticalAlerts = activeAlerts.filter(alert => alert.level === 'critical' || alert.level === 'error');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">System Monitoring</h1>
          <p className="text-gray-600">Real-time infrastructure health and performance</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <Badge variant={autoRefresh ? "default" : "outline"} className="animate-pulse">
            {autoRefresh ? 'Auto-refresh ON' : 'Auto-refresh OFF'}
          </Badge>
          
          <Button 
            variant="outline" 
            size="sm" 
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            <ZapIcon className="w-4 h-4 mr-2" />
            {autoRefresh ? 'Disable' : 'Enable'} Auto-refresh
          </Button>
          
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

      {/* Critical Alerts Banner */}
      {criticalAlerts.length > 0 && (
        <Card className="border-red-200 bg-red-50">
          <CardContent className="p-4">
            <div className="flex items-center space-x-3">
              <AlertTriangleIcon className="w-6 h-6 text-red-600" />
              <div>
                <h3 className="font-semibold text-red-900">
                  {criticalAlerts.length} Critical Alert{criticalAlerts.length > 1 ? 's' : ''}
                </h3>
                <p className="text-sm text-red-700">
                  Immediate attention required for system components
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* System Status Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <ServerIcon className="w-5 h-5 text-blue-600" />
                <span className="font-medium">API Gateway</span>
              </div>
              <Badge variant={getStatusColor(metrics.api_gateway.status)}>
                {getStatusIcon(metrics.api_gateway.status)}
                <span className="ml-1 capitalize">{metrics.api_gateway.status}</span>
              </Badge>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Response Time:</span>
                <span>{metrics.api_gateway.response_time}ms</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Requests/min:</span>
                <span>{metrics.api_gateway.requests_per_minute}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Error Rate:</span>
                <span>{formatPercentage(metrics.api_gateway.error_rate)}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <DatabaseIcon className="w-5 h-5 text-green-600" />
                <span className="font-medium">Database</span>
              </div>
              <Badge variant={getStatusColor(metrics.database.status)}>
                {getStatusIcon(metrics.database.status)}
                <span className="ml-1 capitalize">{metrics.database.status}</span>
              </Badge>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Connections:</span>
                <span>{metrics.database.connections}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Query Time:</span>
                <span>{metrics.database.query_time_avg}ms</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Storage:</span>
                <span>{formatBytes(metrics.database.storage_used)} / {formatBytes(metrics.database.storage_total)}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <HardDriveIcon className="w-5 h-5 text-orange-600" />
                <span className="font-medium">Redis Cache</span>
              </div>
              <Badge variant={getStatusColor(metrics.redis.status)}>
                {getStatusIcon(metrics.redis.status)}
                <span className="ml-1 capitalize">{metrics.redis.status}</span>
              </Badge>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Memory:</span>
                <span>{formatBytes(metrics.redis.memory_used)} / {formatBytes(metrics.redis.memory_total)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Keys:</span>
                <span>{metrics.redis.keys_count.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Hit Rate:</span>
                <span>{formatPercentage(metrics.redis.hit_rate)}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <WifiIcon className="w-5 h-5 text-purple-600" />
                <span className="font-medium">Blockchain</span>
              </div>
              <Badge variant={getStatusColor(metrics.blockchain.status)}>
                {getStatusIcon(metrics.blockchain.status)}
                <span className="ml-1 capitalize">{metrics.blockchain.status}</span>
              </Badge>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Latest Block:</span>
                <span>{metrics.blockchain.latest_block.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Sync Status:</span>
                <Badge variant={metrics.blockchain.sync_status === 'synced' ? 'success' : 'warning'}>
                  {metrics.blockchain.sync_status}
                </Badge>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">TX Pool:</span>
                <span>{metrics.blockchain.transaction_pool}</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Node Status */}
      <Card>
        <CardHeader>
          <CardTitle>Node Network Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600">{metrics.nodes.total}</div>
              <div className="text-sm text-gray-600">Total Nodes</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-600">{metrics.nodes.active}</div>
              <div className="text-sm text-gray-600">Active Nodes</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-orange-600">{metrics.nodes.processing}</div>
              <div className="text-sm text-gray-600">Processing Tasks</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-red-600">{metrics.nodes.offline}</div>
              <div className="text-sm text-gray-600">Offline Nodes</div>
            </div>
          </div>
          
          <div className="mt-6">
            <div className="flex justify-between text-sm mb-2">
              <span>Average Node Load</span>
              <span>{formatPercentage(metrics.nodes.avg_load)}</span>
            </div>
            <Progress value={metrics.nodes.avg_load} className="h-2" />
          </div>
        </CardContent>
      </Card>

      {/* Resource Usage Chart */}
      <Card>
        <CardHeader>
          <CardTitle>Resource Usage Trends</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={metrics.resource_usage}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="timestamp" 
                  tickFormatter={(value) => format(new Date(value), 'HH:mm')}
                />
                <YAxis 
                  domain={[0, 100]}
                  tickFormatter={(value) => `${value}%`}
                />
                <Tooltip 
                  labelFormatter={(value) => format(new Date(value), 'PPpp')}
                  formatter={(value, name) => [
                    `${Number(value).toFixed(1)}%`,
                    name === 'cpu_usage' ? 'CPU Usage' :
                    name === 'memory_usage' ? 'Memory Usage' :
                    name === 'disk_usage' ? 'Disk Usage' : 'Network I/O'
                  ]}
                />
                <Area 
                  type="monotone" 
                  dataKey="cpu_usage" 
                  stackId="1"
                  stroke="#8884d8" 
                  fill="#8884d8" 
                  fillOpacity={0.6}
                />
                <Area 
                  type="monotone" 
                  dataKey="memory_usage" 
                  stackId="2"
                  stroke="#82ca9d" 
                  fill="#82ca9d" 
                  fillOpacity={0.6}
                />
                <Area 
                  type="monotone" 
                  dataKey="disk_usage" 
                  stackId="3"
                  stroke="#ffc658" 
                  fill="#ffc658" 
                  fillOpacity={0.6}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>

      {/* Active Alerts */}
      <Card>
        <CardHeader>
          <CardTitle>
            System Alerts 
            {activeAlerts.length > 0 && (
              <Badge variant="destructive" className="ml-2">
                {activeAlerts.length} Active
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {activeAlerts.length === 0 ? (
            <div className="text-center text-gray-500 py-8">
              <CheckCircleIcon className="w-12 h-12 mx-auto mb-4 text-green-500" />
              <h3 className="text-lg font-medium mb-2">All systems operational</h3>
              <p>No active alerts at this time</p>
            </div>
          ) : (
            <div className="space-y-3">
              {activeAlerts.slice(0, 10).map((alert) => (
                <div 
                  key={alert.id}
                  className={`p-4 rounded-lg border ${
                    alert.level === 'critical' ? 'bg-red-50 border-red-200' :
                    alert.level === 'error' ? 'bg-red-50 border-red-200' :
                    alert.level === 'warning' ? 'bg-yellow-50 border-yellow-200' :
                    'bg-blue-50 border-blue-200'
                  }`}
                >
                  <div className="flex items-start space-x-3">
                    {getAlertIcon(alert.level)}
                    <div className="flex-1">
                      <div className="flex items-center justify-between">
                        <Badge variant={
                          alert.level === 'critical' ? 'destructive' :
                          alert.level === 'error' ? 'destructive' :
                          alert.level === 'warning' ? 'default' : 'secondary'
                        }>
                          {alert.level.toUpperCase()}
                        </Badge>
                        <span className="text-xs text-gray-500">
                          {formatDistanceToNow(new Date(alert.timestamp), { addSuffix: true })}
                        </span>
                      </div>
                      <p className="mt-1 text-sm">{alert.message}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}