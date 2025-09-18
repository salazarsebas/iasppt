import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useMutation, useQueryClient } from 'react-query';
import { toast } from 'react-hot-toast';
import { apiClient } from '@/lib/api';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Label } from '@/components/ui/Label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { PlayIcon, InfoIcon } from 'lucide-react';

const taskSchema = yup.object({
  task_type: yup.string().required('Task type is required'),
  model_name: yup.string().required('Model name is required'),
  input_data: yup.string()
    .required('Input data is required')
    .max(50000, 'Input data too large (max 50,000 characters)'),
  max_cost: yup.string().optional(),
  priority: yup.number().min(0).max(10).optional(),
});

type TaskFormData = yup.InferType<typeof taskSchema>;

interface TaskTemplate {
  name: string;
  description: string;
  task_type: string;
  model_name: string;
  example_input: string;
  estimated_cost: string;
  parameters?: Record<string, any>;
}

const TASK_TEMPLATES: TaskTemplate[] = [
  {
    name: 'Text Classification',
    description: 'Classify text into categories (sentiment, topics, etc.)',
    task_type: 'classification',
    model_name: 'cardiffnlp/twitter-roberta-base-sentiment-latest',
    example_input: 'I love this new AI technology! It\'s amazing how fast it processes data.',
    estimated_cost: '0.02 NEAR',
  },
  {
    name: 'Text Generation',
    description: 'Generate text using large language models',
    task_type: 'text_generation',
    model_name: 'gpt2',
    example_input: 'The future of artificial intelligence is',
    estimated_cost: '0.05 NEAR',
    parameters: { max_length: 100, temperature: 0.7 },
  },
  {
    name: 'Text Embedding',
    description: 'Convert text to numerical embeddings for semantic search',
    task_type: 'embedding',
    model_name: 'sentence-transformers/all-MiniLM-L6-v2',
    example_input: 'DeAI is a decentralized AI computation network.',
    estimated_cost: '0.015 NEAR',
  },
  {
    name: 'Question Answering',
    description: 'Answer questions based on context',
    task_type: 'inference',
    model_name: 'deepset/roberta-base-squad2',
    example_input: JSON.stringify({
      question: 'What is DeAI?',
      context: 'DeAI is a decentralized AI computation network that allows users to submit AI tasks and get results from distributed GPU nodes powered by Near Protocol.'
    }),
    estimated_cost: '0.03 NEAR',
  },
];

export function TaskSubmissionForm() {
  const [selectedTemplate, setSelectedTemplate] = useState<TaskTemplate | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const queryClient = useQueryClient();

  const { register, handleSubmit, setValue, watch, formState: { errors } } = useForm<TaskFormData>({
    resolver: yupResolver(taskSchema),
    defaultValues: {
      priority: 5,
    },
  });

  const watchedValues = watch();

  const submitTaskMutation = useMutation(
    (data: TaskFormData) => apiClient.submitTask(data),
    {
      onSuccess: (task) => {
        toast.success(`Task submitted successfully! ID: ${task.id}`);
        queryClient.invalidateQueries('user-tasks');
        // Reset form or redirect
      },
      onError: (error: any) => {
        toast.error(error.response?.data?.message || 'Failed to submit task');
      },
    }
  );

  const handleTemplateSelect = (template: TaskTemplate) => {
    setSelectedTemplate(template);
    setValue('task_type', template.task_type);
    setValue('model_name', template.model_name);
    setValue('input_data', template.example_input);
  };

  const onSubmit = (data: TaskFormData) => {
    const parameters = selectedTemplate?.parameters;
    submitTaskMutation.mutate({
      ...data,
      parameters: parameters ? parameters : undefined,
    });
  };

  const estimatedCost = selectedTemplate?.estimated_cost || 'Calculating...';

  return (
    <div className="space-y-6">
      {/* Template Selection */}
      <div>
        <Label className="text-base font-semibold">Choose a Template</Label>
        <p className="text-sm text-gray-600 mb-4">
          Select a pre-configured task template or create a custom task below.
        </p>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {TASK_TEMPLATES.map((template) => (
            <Card 
              key={template.name}
              className={`cursor-pointer transition-all hover:shadow-md ${
                selectedTemplate?.name === template.name 
                  ? 'ring-2 ring-blue-500 border-blue-500' 
                  : 'hover:border-gray-300'
              }`}
              onClick={() => handleTemplateSelect(template)}
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{template.name}</CardTitle>
                  <Badge variant="secondary">{template.estimated_cost}</Badge>
                </div>
                <CardDescription className="text-sm">
                  {template.description}
                </CardDescription>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="text-xs text-gray-500">
                  <p><strong>Model:</strong> {template.model_name}</p>
                  <p><strong>Type:</strong> {template.task_type}</p>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>

      {/* Task Form */}
      <Card>
        <CardHeader>
          <CardTitle>Task Configuration</CardTitle>
          <CardDescription>
            Configure your AI task parameters below.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            {/* Task Type */}
            <div>
              <Label htmlFor="task_type">Task Type *</Label>
              <Select 
                value={watchedValues.task_type} 
                onValueChange={(value) => setValue('task_type', value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select task type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="inference">Inference</SelectItem>
                  <SelectItem value="text_generation">Text Generation</SelectItem>
                  <SelectItem value="classification">Classification</SelectItem>
                  <SelectItem value="embedding">Embedding</SelectItem>
                </SelectContent>
              </Select>
              {errors.task_type && (
                <p className="text-sm text-red-600 mt-1">{errors.task_type.message}</p>
              )}
            </div>

            {/* Model Name */}
            <div>
              <Label htmlFor="model_name">Model Name *</Label>
              <Input
                {...register('model_name')}
                placeholder="e.g., gpt2, bert-base-uncased"
                className={errors.model_name ? 'border-red-500' : ''}
              />
              {errors.model_name && (
                <p className="text-sm text-red-600 mt-1">{errors.model_name.message}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Use Hugging Face model names or custom model identifiers
              </p>
            </div>

            {/* Input Data */}
            <div>
              <Label htmlFor="input_data">Input Data *</Label>
              <Textarea
                {...register('input_data')}
                placeholder="Enter your input text or JSON data..."
                className={`min-h-32 ${errors.input_data ? 'border-red-500' : ''}`}
              />
              {errors.input_data && (
                <p className="text-sm text-red-600 mt-1">{errors.input_data.message}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Character count: {watchedValues.input_data?.length || 0} / 50,000
              </p>
            </div>

            {/* Advanced Options */}
            <div>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="mb-4"
              >
                {showAdvanced ? 'Hide' : 'Show'} Advanced Options
              </Button>

              {showAdvanced && (
                <div className="space-y-4 p-4 bg-gray-50 rounded-lg">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <Label htmlFor="priority">Priority (0-10)</Label>
                      <Input
                        {...register('priority')}
                        type="number"
                        min="0"
                        max="10"
                        placeholder="5"
                      />
                      <p className="text-xs text-gray-500 mt-1">
                        Higher priority tasks are processed first
                      </p>
                    </div>

                    <div>
                      <Label htmlFor="max_cost">Max Cost (NEAR)</Label>
                      <Input
                        {...register('max_cost')}
                        placeholder="0.1"
                        step="0.001"
                      />
                      <p className="text-xs text-gray-500 mt-1">
                        Task will fail if cost exceeds this amount
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Cost Estimate */}
            <Card className="bg-blue-50 border-blue-200">
              <CardContent className="pt-4">
                <div className="flex items-center space-x-2">
                  <InfoIcon className="w-4 h-4 text-blue-600" />
                  <span className="text-sm font-medium text-blue-900">
                    Estimated Cost: {estimatedCost}
                  </span>
                </div>
                <p className="text-xs text-blue-700 mt-1">
                  Final cost may vary based on actual computation time and resources used.
                </p>
              </CardContent>
            </Card>

            {/* Submit Button */}
            <div className="flex justify-end space-x-3">
              <Button 
                type="button" 
                variant="outline"
                onClick={() => {
                  setSelectedTemplate(null);
                  setValue('task_type', '');
                  setValue('model_name', '');
                  setValue('input_data', '');
                  setValue('max_cost', '');
                  setValue('priority', 5);
                }}
              >
                Reset
              </Button>
              
              <Button 
                type="submit" 
                disabled={submitTaskMutation.isLoading}
                className="min-w-32"
              >
                {submitTaskMutation.isLoading ? (
                  <LoadingSpinner size="sm" />
                ) : (
                  <>
                    <PlayIcon className="w-4 h-4 mr-2" />
                    Submit Task
                  </>
                )}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}