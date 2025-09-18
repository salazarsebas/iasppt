import { useState, useEffect } from 'react';
import Head from 'next/head';
import { useWalletSelector } from '@/hooks/useWalletSelector';
import { useAuth } from '@/hooks/useAuth';
import { Header } from '@/components/Header';
import { TaskSubmissionForm } from '@/components/TaskSubmissionForm';
import { TaskList } from '@/components/TaskList';
import { NetworkStats } from '@/components/NetworkStats';
import { WalletConnection } from '@/components/WalletConnection';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { PlusIcon, BarChart3Icon, Settings2Icon } from 'lucide-react';

export default function Dashboard() {
  const { isConnected, accountId } = useWalletSelector();
  const { user, isAuthenticated, login, logout } = useAuth();
  const [activeTab, setActiveTab] = useState('tasks');

  useEffect(() => {
    if (isConnected && !isAuthenticated) {
      // Auto-login with Near wallet if connected but not authenticated
      login({ type: 'near', accountId });
    }
  }, [isConnected, isAuthenticated, accountId, login]);

  if (!isConnected) {
    return (
      <>
        <Head>
          <title>DeAI Dashboard - Connect Wallet</title>
          <meta name="description" content="Connect your Near wallet to access DeAI compute network" />
        </Head>
        
        <div className="min-h-screen bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900">
          <div className="container mx-auto px-4 py-16">
            <div className="text-center">
              <h1 className="text-6xl font-bold text-white mb-6">
                DeAI
                <span className="bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
                  {' '}Compute
                </span>
              </h1>
              
              <p className="text-xl text-gray-300 mb-12 max-w-2xl mx-auto">
                Decentralized AI computation network powered by Near Protocol. 
                Submit AI tasks and get results from distributed GPU nodes.
              </p>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-12 max-w-4xl mx-auto">
                <Card className="bg-white/10 backdrop-blur-lg border-white/20">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <PlusIcon className="w-5 h-5 mr-2" />
                      Submit Tasks
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-gray-300">
                      Submit AI inference, text generation, and classification tasks
                    </p>
                  </CardContent>
                </Card>
                
                <Card className="bg-white/10 backdrop-blur-lg border-white/20">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <BarChart3Icon className="w-5 h-5 mr-2" />
                      Monitor Progress
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-gray-300">
                      Track task status and view detailed execution results
                    </p>
                  </CardContent>
                </Card>
                
                <Card className="bg-white/10 backdrop-blur-lg border-white/20">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Settings2Icon className="w-5 h-5 mr-2" />
                      Pay with NEAR
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-gray-300">
                      Transparent pricing with instant payments via Near Protocol
                    </p>
                  </CardContent>
                </Card>
              </div>
              
              <WalletConnection />
            </div>
          </div>
        </div>
      </>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <>
      <Head>
        <title>DeAI Dashboard - AI Task Management</title>
        <meta name="description" content="Manage your AI computation tasks on the DeAI network" />
      </Head>

      <div className="min-h-screen bg-gray-50">
        <Header user={user} onLogout={logout} />
        
        <main className="container mx-auto px-4 py-8">
          {/* Welcome Section */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">
              Welcome back, {user?.username}
            </h1>
            <p className="text-gray-600">
              Connected as <Badge variant="outline">{accountId}</Badge>
            </p>
          </div>

          {/* Network Stats Overview */}
          <NetworkStats className="mb-8" />

          {/* Main Content Tabs */}
          <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="tasks">My Tasks</TabsTrigger>
              <TabsTrigger value="submit">Submit Task</TabsTrigger>
              <TabsTrigger value="analytics">Analytics</TabsTrigger>
              <TabsTrigger value="settings">Settings</TabsTrigger>
            </TabsList>

            <TabsContent value="tasks" className="mt-6">
              <Card>
                <CardHeader>
                  <CardTitle>Task Management</CardTitle>
                  <CardDescription>
                    View and manage your AI computation tasks
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <TaskList />
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="submit" className="mt-6">
              <Card>
                <CardHeader>
                  <CardTitle>Submit AI Task</CardTitle>
                  <CardDescription>
                    Create a new AI computation task for the network
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <TaskSubmissionForm />
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="analytics" className="mt-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle>Usage Statistics</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <UserAnalytics />
                  </CardContent>
                </Card>
                
                <Card>
                  <CardHeader>
                    <CardTitle>Cost Analysis</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CostAnalytics />
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            <TabsContent value="settings" className="mt-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle>Account Settings</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <AccountSettings />
                  </CardContent>
                </Card>
                
                <Card>
                  <CardHeader>
                    <CardTitle>API Keys</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ApiKeyManagement />
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
          </Tabs>
        </main>
      </div>
    </>
  );
}

// Component imports would be defined elsewhere
function UserAnalytics() {
  return <div>User analytics component placeholder</div>;
}

function CostAnalytics() {
  return <div>Cost analytics component placeholder</div>;
}

function AccountSettings() {
  return <div>Account settings component placeholder</div>;
}

function ApiKeyManagement() {
  return <div>API key management component placeholder</div>;
}