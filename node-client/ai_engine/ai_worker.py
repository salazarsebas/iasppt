#!/usr/bin/env python3
"""
DeAI AI Processing Engine - Python Worker
Handles AI model execution for the DeAI compute network
"""

import os
import sys
import json
import logging
import hashlib
import tempfile
import time
from typing import Dict, Any, Tuple, Optional
from pathlib import Path

import torch
import transformers
from transformers import (
    AutoTokenizer, AutoModel, AutoModelForCausalLM, 
    AutoModelForSequenceClassification, pipeline
)
from huggingface_hub import login, hf_hub_download
import psutil
import GPUtil

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class AIWorker:
    """AI processing worker for executing tasks"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.models_cache = Path(config.get('models_cache_dir', './models_cache'))
        self.models_cache.mkdir(exist_ok=True)
        
        # Initialize Hugging Face
        hf_token = config.get('huggingface_token')
        if hf_token:
            login(token=hf_token)
            logger.info("Logged into Hugging Face")
        
        # Check hardware
        self.check_hardware()
        
        logger.info(f"AI Worker initialized. Cache dir: {self.models_cache}")
    
    def check_hardware(self):
        """Check available hardware resources"""
        # CPU info
        cpu_count = psutil.cpu_count()
        memory = psutil.virtual_memory()
        logger.info(f"CPU cores: {cpu_count}, Memory: {memory.total / (1024**3):.1f} GB")
        
        # GPU info
        if torch.cuda.is_available():
            gpu_count = torch.cuda.device_count()
            logger.info(f"CUDA available with {gpu_count} GPU(s)")
            for i in range(gpu_count):
                gpu = torch.cuda.get_device_properties(i)
                logger.info(f"GPU {i}: {gpu.name}, {gpu.total_memory / (1024**3):.1f} GB")
        else:
            logger.warning("CUDA not available, using CPU")
    
    async def execute_task(self, task_data: Dict[str, Any]) -> Tuple[str, str]:
        """
        Execute an AI task and return (proof_hash, output)
        
        Args:
            task_data: Task description containing model, input, and parameters
            
        Returns:
            Tuple of (proof_hash, output_json)
        """
        try:
            logger.info(f"Executing task: {task_data.get('id', 'unknown')}")
            start_time = time.time()
            
            # Parse task description
            if isinstance(task_data.get('description'), str):
                task_desc = json.loads(task_data['description'])
            else:
                task_desc = task_data['description']
            
            task_type = task_desc.get('task_type', 'inference')
            model_name = task_desc.get('model', '')
            input_data = task_desc.get('input', '')
            parameters = task_desc.get('parameters', {})
            
            logger.info(f"Task type: {task_type}, Model: {model_name}")
            
            # Execute based on task type
            if task_type == 'inference':
                result = await self._run_inference(model_name, input_data, parameters)
            elif task_type == 'text_generation':
                result = await self._run_text_generation(model_name, input_data, parameters)
            elif task_type == 'classification':
                result = await self._run_classification(model_name, input_data, parameters)
            elif task_type == 'embedding':
                result = await self._run_embedding(model_name, input_data, parameters)
            else:
                raise ValueError(f"Unsupported task type: {task_type}")
            
            execution_time = time.time() - start_time
            
            # Prepare output
            output = {
                'result': result,
                'execution_time': execution_time,
                'model': model_name,
                'task_type': task_type,
                'timestamp': time.time(),
                'hardware_info': self._get_hardware_info()
            }
            
            output_json = json.dumps(output, ensure_ascii=False)
            proof_hash = self._generate_proof_hash(output_json)
            
            logger.info(f"Task completed in {execution_time:.2f}s")
            return proof_hash, output_json
            
        except Exception as e:
            logger.error(f"Task execution failed: {e}")
            error_output = {
                'error': str(e),
                'timestamp': time.time(),
                'task_type': task_data.get('task_type', 'unknown')
            }
            error_json = json.dumps(error_output)
            error_hash = self._generate_proof_hash(error_json)
            return error_hash, error_json
    
    async def _run_inference(self, model_name: str, input_data: str, parameters: Dict) -> Any:
        """Run general model inference"""
        model, tokenizer = await self._load_model(model_name)
        
        if tokenizer:
            inputs = tokenizer(input_data, return_tensors="pt", truncation=True, max_length=512)
            
            with torch.no_grad():
                outputs = model(**inputs)
                
            if hasattr(outputs, 'logits'):
                predictions = torch.softmax(outputs.logits, dim=-1)
                return predictions.tolist()
            else:
                return outputs.last_hidden_state.mean(dim=1).tolist()
        else:
            # For models without tokenizer, treat as image or other data
            return {"message": "Model loaded but no specific inference implemented"}
    
    async def _run_text_generation(self, model_name: str, input_data: str, parameters: Dict) -> str:
        """Run text generation"""
        generator = pipeline(
            "text-generation",
            model=model_name,
            device=0 if torch.cuda.is_available() else -1,
            cache_dir=str(self.models_cache)
        )
        
        max_length = parameters.get('max_length', 100)
        temperature = parameters.get('temperature', 0.7)
        
        result = generator(
            input_data,
            max_length=max_length,
            temperature=temperature,
            do_sample=True,
            num_return_sequences=1
        )
        
        return result[0]['generated_text']
    
    async def _run_classification(self, model_name: str, input_data: str, parameters: Dict) -> Dict:
        """Run text classification"""
        classifier = pipeline(
            "text-classification",
            model=model_name,
            device=0 if torch.cuda.is_available() else -1,
            cache_dir=str(self.models_cache)
        )
        
        result = classifier(input_data)
        return result
    
    async def _run_embedding(self, model_name: str, input_data: str, parameters: Dict) -> list:
        """Generate embeddings"""
        model, tokenizer = await self._load_model(model_name)
        
        inputs = tokenizer(input_data, return_tensors="pt", truncation=True, max_length=512)
        
        with torch.no_grad():
            outputs = model(**inputs)
            embeddings = outputs.last_hidden_state.mean(dim=1)
        
        return embeddings.squeeze().tolist()
    
    async def _load_model(self, model_name: str) -> Tuple[Any, Any]:
        """Load model and tokenizer"""
        cache_dir = self.models_cache / model_name.replace('/', '_')
        
        try:
            logger.info(f"Loading model: {model_name}")
            
            # Try to load tokenizer
            try:
                tokenizer = AutoTokenizer.from_pretrained(
                    model_name,
                    cache_dir=str(cache_dir)
                )
            except Exception as e:
                logger.warning(f"Could not load tokenizer for {model_name}: {e}")
                tokenizer = None
            
            # Try different model classes
            model_classes = [
                AutoModelForCausalLM,
                AutoModelForSequenceClassification,
                AutoModel
            ]
            
            model = None
            for model_class in model_classes:
                try:
                    model = model_class.from_pretrained(
                        model_name,
                        cache_dir=str(cache_dir),
                        torch_dtype=torch.float16 if torch.cuda.is_available() else torch.float32,
                        device_map="auto" if torch.cuda.is_available() else None
                    )
                    break
                except Exception as e:
                    logger.debug(f"Failed to load with {model_class.__name__}: {e}")
            
            if model is None:
                raise ValueError(f"Could not load model {model_name} with any model class")
            
            logger.info(f"Successfully loaded model: {model_name}")
            return model, tokenizer
            
        except Exception as e:
            logger.error(f"Failed to load model {model_name}: {e}")
            raise
    
    def _generate_proof_hash(self, output: str) -> str:
        """Generate proof hash for the output"""
        timestamp = str(int(time.time()))
        node_id = self.config.get('node_id', 'unknown')
        
        proof_data = f"{output}:{timestamp}:{node_id}"
        return hashlib.sha256(proof_data.encode()).hexdigest()
    
    def _get_hardware_info(self) -> Dict[str, Any]:
        """Get current hardware usage info"""
        cpu_percent = psutil.cpu_percent()
        memory = psutil.virtual_memory()
        
        info = {
            'cpu_percent': cpu_percent,
            'memory_percent': memory.percent,
            'memory_available_gb': memory.available / (1024**3)
        }
        
        if torch.cuda.is_available():
            gpus = GPUtil.getGPUs()
            if gpus:
                gpu = gpus[0]  # Use first GPU
                info.update({
                    'gpu_name': gpu.name,
                    'gpu_memory_percent': gpu.memoryUtil * 100,
                    'gpu_temperature': gpu.temperature
                })
        
        return info

def main():
    """Main entry point for standalone execution"""
    if len(sys.argv) != 2:
        print("Usage: python ai_worker.py <task_json>")
        sys.exit(1)
    
    task_json = sys.argv[1]
    task_data = json.loads(task_json)
    
    # Default config for standalone execution
    config = {
        'models_cache_dir': './models_cache',
        'node_id': 'standalone'
    }
    
    worker = AIWorker(config)
    
    # Run synchronously for CLI usage
    import asyncio
    proof_hash, output = asyncio.run(worker.execute_task(task_data))
    
    result = {
        'proof_hash': proof_hash,
        'output': output
    }
    
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()