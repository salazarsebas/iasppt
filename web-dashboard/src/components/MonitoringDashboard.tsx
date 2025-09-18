import React, { useState, useEffect, useCallback } from 'react';
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  Alert,
  CircularProgress,
  Chip,
  LinearProgress,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  IconButton,
  Tooltip,
  Switch,
  FormControlLabel
} from '@mui/material';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine
} from 'recharts';
import {
  Computer,
  Memory,
  Speed,
  AccountBalance,
  TrendingUp,
  Warning,
  CheckCircle,
  Error,
  Refresh,
  Visibility,
  VisibilityOff,
  Timeline,
  Analytics
} from '@mui/icons-material';

interface NodeMetrics {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'maintenance';
  cpuUsage: number;
  memoryUsage: number;
  gpuUsage: number;
  tasksCompleted: number;
  tasksInProgress: number;
  lastHeartbeat: string;
  uptime: number;
  reputationScore: number;
  earnings: number;
  location: string;
  version: string;
}

interface TaskMetrics {
  timestamp: string;
  totalTasks: number;
  pendingTasks: number;
  runningTasks: number;
  completedTasks: number;
  failedTasks: number;
  avgProcessingTime: number;
  throughput: number;
}

interface TokenMetrics {
  totalSupply: number;
  circulatingSupply: number;
  totalRewardsDistributed: number;
  dailyVolume: number;
  price: number;
  marketCap: number;
  holders: number;
  burnedTokens: number;
}

interface SystemMetrics {
  activeNodes: number;
  totalNodes: number;
  networkHashrate: number;
  avgResponseTime: number;
  errorRate: number;
  successRate: number;
  queueLength: number;
  systemLoad: number;
}

interface AlertItem {
  id: string;
  severity: 'error' | 'warning' | 'info';
  message: string;
  timestamp: string;
  nodeId?: string;
  resolved: boolean;
}

const MonitoringDashboard: React.FC = () => {
  const [nodeMetrics, setNodeMetrics] = useState<NodeMetrics[]>([]);
  const [taskMetrics, setTaskMetrics] = useState<TaskMetrics[]>([]);
  const [tokenMetrics, setTokenMetrics] = useState<TokenMetrics | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [alerts, setAlerts] = useState<AlertItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selectedTimeRange, setSelectedTimeRange] = useState('24h');
  const [error, setError] = useState<string | null>(null);

  // Fetch all metrics data
  const fetchMetrics = useCallback(async () => {
    try {
      setError(null);
      
      // In a real implementation, these would be API calls
      const [nodes, tasks, tokens, system, alertsData] = await Promise.all([
        fetchNodeMetrics(),
        fetchTaskMetrics(selectedTimeRange),
        fetchTokenMetrics(),
        fetchSystemMetrics(),
        fetchAlerts()
      ]);

      setNodeMetrics(nodes);
      setTaskMetrics(tasks);
      setTokenMetrics(tokens);
      setSystemMetrics(system);
      setAlerts(alertsData);
      setLoading(false);
    } catch (err) {
      setError('Failed to fetch metrics data');
      setLoading(false);
    }
  }, [selectedTimeRange]);

  useEffect(() => {
    fetchMetrics();
  }, [fetchMetrics]);

  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(fetchMetrics, 30000); // Refresh every 30 seconds
      return () => clearInterval(interval);
    }
  }, [autoRefresh, fetchMetrics]);

  const handleRefresh = () => {
    setLoading(true);
    fetchMetrics();
  };

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="400px">
        <CircularProgress size={60} />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" action={
        <IconButton color="inherit" size="small" onClick={handleRefresh}>
          <Refresh />
        </IconButton>
      }>
        {error}
      </Alert>
    );
  }

  return (
    <Box sx={{ flexGrow: 1, p: 3 }}>
      {/* Header */}
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h4" component="h1">
          DeAI Network Monitoring Dashboard
        </Typography>
        <Box display="flex" alignItems="center" gap={2}>
          <FormControlLabel
            control={
              <Switch
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
              />
            }
            label="Auto Refresh"
          />
          <IconButton onClick={handleRefresh} disabled={loading}>
            <Refresh />
          </IconButton>
        </Box>
      </Box>

      {/* System Overview Cards */}
      <Grid container spacing={3} mb={3}>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Active Nodes"
            value={systemMetrics?.activeNodes || 0}
            total={systemMetrics?.totalNodes || 0}
            icon={<Computer />}
            color="primary"
            suffix={`/${systemMetrics?.totalNodes || 0}`}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Success Rate"
            value={systemMetrics?.successRate || 0}
            icon={<CheckCircle />}
            color="success"
            suffix="%"
            showProgress={true}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Queue Length"
            value={systemMetrics?.queueLength || 0}
            icon={<Timeline />}
            color="warning"
            trend={getQueueTrend()}
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Avg Response Time"
            value={systemMetrics?.avgResponseTime || 0}
            icon={<Speed />}
            color="info"
            suffix="ms"
          />
        </Grid>
      </Grid>

      {/* Alerts Section */}
      {alerts.filter(a => !a.resolved).length > 0 && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Active Alerts
            </Typography>
            {alerts.filter(a => !a.resolved).map((alert) => (
              <Alert
                key={alert.id}
                severity={alert.severity}
                sx={{ mb: 1 }}
                action={
                  <Chip
                    label={new Date(alert.timestamp).toLocaleTimeString()}
                    size="small"
                    variant="outlined"
                  />
                }
              >
                {alert.message}
                {alert.nodeId && ` (Node: ${alert.nodeId})`}
              </Alert>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Charts Section */}
      <Grid container spacing={3} mb={3}>
        {/* Task Processing Chart */}
        <Grid item xs={12} md={8}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Task Processing Over Time
              </Typography>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={taskMetrics}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="timestamp" />
                  <YAxis />
                  <RechartsTooltip />
                  <Legend />
                  <Area
                    type="monotone"
                    dataKey="completedTasks"
                    stackId="1"
                    stroke="#4CAF50"
                    fill="#4CAF50"
                    name="Completed"
                  />
                  <Area
                    type="monotone"
                    dataKey="runningTasks"
                    stackId="1"
                    stroke="#FF9800"
                    fill="#FF9800"
                    name="Running"
                  />
                  <Area
                    type="monotone"
                    dataKey="pendingTasks"
                    stackId="1"
                    stroke="#2196F3"
                    fill="#2196F3"
                    name="Pending"
                  />
                  <Area
                    type="monotone"
                    dataKey="failedTasks"
                    stackId="1"
                    stroke="#F44336"
                    fill="#F44336"
                    name="Failed"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* Node Status Distribution */}
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Node Status Distribution
              </Typography>
              <ResponsiveContainer width="100%" height={300}>
                <PieChart>
                  <Pie
                    data={getNodeStatusData()}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                    outerRadius={80}
                    fill="#8884d8"
                    dataKey="value"
                  >
                    {getNodeStatusData().map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <RechartsTooltip />
                </PieChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Token Economics */}
      <Grid container spacing={3} mb={3}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Token Economics
              </Typography>
              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Total Supply
                    </Typography>
                    <Typography variant="h6">
                      {formatNumber(tokenMetrics?.totalSupply || 0)} DEAI
                    </Typography>
                  </Box>
                </Grid>
                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Circulating Supply
                    </Typography>
                    <Typography variant="h6">
                      {formatNumber(tokenMetrics?.circulatingSupply || 0)} DEAI
                    </Typography>
                  </Box>
                </Grid>
                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Rewards Distributed
                    </Typography>
                    <Typography variant="h6">
                      {formatNumber(tokenMetrics?.totalRewardsDistributed || 0)} DEAI
                    </Typography>
                  </Box>
                </Grid>
                <Grid item xs={6}>
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Token Holders
                    </Typography>
                    <Typography variant="h6">
                      {formatNumber(tokenMetrics?.holders || 0)}
                    </Typography>
                  </Box>
                </Grid>
              </Grid>
            </CardContent>
          </Card>
        </Grid>

        {/* Throughput Chart */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Network Throughput (TPS)
              </Typography>
              <ResponsiveContainer width="100%" height={200}>
                <LineChart data={taskMetrics}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="timestamp" />
                  <YAxis />
                  <RechartsTooltip />
                  <Line
                    type="monotone"
                    dataKey="throughput"
                    stroke="#2196F3"
                    strokeWidth={2}
                    dot={{ r: 4 }}
                  />
                  <ReferenceLine y={4000} stroke="red" strokeDasharray="5 5" label="Target: 4000 TPS" />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Node Details Table */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Node Details
          </Typography>
          <TableContainer component={Paper} variant="outlined">
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Node ID</TableCell>
                  <TableCell>Status</TableCell>
                  <TableCell>CPU</TableCell>
                  <TableCell>Memory</TableCell>
                  <TableCell>GPU</TableCell>
                  <TableCell>Tasks</TableCell>
                  <TableCell>Reputation</TableCell>
                  <TableCell>Earnings</TableCell>
                  <TableCell>Uptime</TableCell>
                  <TableCell>Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {nodeMetrics.map((node) => (
                  <TableRow key={node.id}>
                    <TableCell>
                      <Typography variant="body2" fontWeight="bold">
                        {node.name}
                      </Typography>
                      <Typography variant="caption" color="textSecondary">
                        {node.id.substring(0, 8)}...
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={node.status}
                        color={getStatusColor(node.status)}
                        size="small"
                      />
                    </TableCell>
                    <TableCell>
                      <Box display="flex" alignItems="center">
                        <LinearProgress
                          variant="determinate"
                          value={node.cpuUsage}
                          sx={{ width: 60, mr: 1 }}
                        />
                        <Typography variant="caption">
                          {node.cpuUsage}%
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Box display="flex" alignItems="center">
                        <LinearProgress
                          variant="determinate"
                          value={node.memoryUsage}
                          sx={{ width: 60, mr: 1 }}
                        />
                        <Typography variant="caption">
                          {node.memoryUsage}%
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Box display="flex" alignItems="center">
                        <LinearProgress
                          variant="determinate"
                          value={node.gpuUsage}
                          sx={{ width: 60, mr: 1 }}
                        />
                        <Typography variant="caption">
                          {node.gpuUsage}%
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">
                        {node.tasksCompleted} / {node.tasksInProgress}
                      </Typography>
                      <Typography variant="caption" color="textSecondary">
                        Completed / Running
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Box display="flex" alignItems="center">
                        <Typography variant="body2">
                          {node.reputationScore}
                        </Typography>
                        {node.reputationScore >= 800 && (
                          <CheckCircle color="success" fontSize="small" sx={{ ml: 0.5 }} />
                        )}
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">
                        {formatNumber(node.earnings)} DEAI
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">
                        {formatUptime(node.uptime)}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Tooltip title="View Details">
                        <IconButton size="small">
                          <Visibility />
                        </IconButton>
                      </Tooltip>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>
    </Box>
  );
};

// Helper Components
interface MetricCardProps {
  title: string;
  value: number;
  total?: number;
  icon: React.ReactNode;
  color: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
  suffix?: string;
  showProgress?: boolean;
  trend?: 'up' | 'down' | 'stable';
}

const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  total,
  icon,
  color,
  suffix = '',
  showProgress = false,
  trend
}) => {
  return (
    <Card>
      <CardContent>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Box>
            <Typography color="textSecondary" gutterBottom variant="body2">
              {title}
            </Typography>
            <Typography variant="h5" component="div">
              {formatNumber(value)}{suffix}
            </Typography>
            {showProgress && (
              <LinearProgress
                variant="determinate"
                value={value}
                color={color}
                sx={{ mt: 1 }}
              />
            )}
          </Box>
          <Box color={`${color}.main`}>
            {icon}
          </Box>
        </Box>
        {trend && (
          <Box display="flex" alignItems="center" mt={1}>
            <TrendingUp
              fontSize="small"
              color={trend === 'up' ? 'success' : trend === 'down' ? 'error' : 'disabled'}
            />
            <Typography variant="caption" color="textSecondary" ml={0.5}>
              {trend === 'up' ? 'Increasing' : trend === 'down' ? 'Decreasing' : 'Stable'}
            </Typography>
          </Box>
        )}
      </CardContent>
    </Card>
  );
};

// Helper Functions
const formatNumber = (num: number): string => {
  if (num >= 1e9) return (num / 1e9).toFixed(1) + 'B';
  if (num >= 1e6) return (num / 1e6).toFixed(1) + 'M';
  if (num >= 1e3) return (num / 1e3).toFixed(1) + 'K';
  return num.toString();
};

const formatUptime = (hours: number): string => {
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return `${days}d ${remainingHours}h`;
};

const getStatusColor = (status: string): 'success' | 'warning' | 'error' => {
  switch (status) {
    case 'online': return 'success';
    case 'maintenance': return 'warning';
    case 'offline': return 'error';
    default: return 'warning';
  }
};

const getQueueTrend = (): 'up' | 'down' | 'stable' => {
  // This would be calculated based on historical data
  return 'stable';
};

const getNodeStatusData = () => [
  { name: 'Online', value: 8, color: '#4CAF50' },
  { name: 'Maintenance', value: 1, color: '#FF9800' },
  { name: 'Offline', value: 1, color: '#F44336' }
];

// Mock API functions (replace with real API calls)
const fetchNodeMetrics = async (): Promise<NodeMetrics[]> => {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 500));
  
  return [
    {
      id: 'node-001',
      name: 'Node-Alpha',
      status: 'online',
      cpuUsage: 75,
      memoryUsage: 60,
      gpuUsage: 85,
      tasksCompleted: 142,
      tasksInProgress: 3,
      lastHeartbeat: new Date().toISOString(),
      uptime: 168,
      reputationScore: 950,
      earnings: 1250.5,
      location: 'US-East',
      version: '1.2.3'
    },
    // Add more mock nodes...
  ];
};

const fetchTaskMetrics = async (timeRange: string): Promise<TaskMetrics[]> => {
  await new Promise(resolve => setTimeout(resolve, 300));
  
  // Generate mock time series data
  const data = [];
  const now = new Date();
  for (let i = 23; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000);
    data.push({
      timestamp: time.toLocaleTimeString(),
      totalTasks: Math.floor(Math.random() * 1000) + 500,
      pendingTasks: Math.floor(Math.random() * 100) + 20,
      runningTasks: Math.floor(Math.random() * 200) + 50,
      completedTasks: Math.floor(Math.random() * 800) + 400,
      failedTasks: Math.floor(Math.random() * 50) + 5,
      avgProcessingTime: Math.random() * 30 + 10,
      throughput: Math.random() * 2000 + 3000
    });
  }
  
  return data;
};

const fetchTokenMetrics = async (): Promise<TokenMetrics> => {
  await new Promise(resolve => setTimeout(resolve, 200));
  
  return {
    totalSupply: 1000000000,
    circulatingSupply: 750000000,
    totalRewardsDistributed: 5000000,
    dailyVolume: 1500000,
    price: 0.85,
    marketCap: 637500000,
    holders: 12500,
    burnedTokens: 2500000
  };
};

const fetchSystemMetrics = async (): Promise<SystemMetrics> => {
  await new Promise(resolve => setTimeout(resolve, 150));
  
  return {
    activeNodes: 8,
    totalNodes: 10,
    networkHashrate: 1500000,
    avgResponseTime: 250,
    errorRate: 0.5,
    successRate: 99.5,
    queueLength: 45,
    systemLoad: 75
  };
};

const fetchAlerts = async (): Promise<AlertItem[]> => {
  await new Promise(resolve => setTimeout(resolve, 100));
  
  return [
    {
      id: 'alert-001',
      severity: 'warning',
      message: 'High CPU usage detected',
      timestamp: new Date().toISOString(),
      nodeId: 'node-003',
      resolved: false
    },
    {
      id: 'alert-002',
      severity: 'info',
      message: 'System maintenance scheduled for tomorrow',
      timestamp: new Date(Date.now() - 60000).toISOString(),
      resolved: false
    }
  ];
};

export default MonitoringDashboard;