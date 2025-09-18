#!/usr/bin/env python3
"""
Example script to test the AI worker with a simple task
"""

import json
import sys
import asyncio
from pathlib import Path

# Add the ai_engine directory to the path
sys.path.append(str(Path(__file__).parent.parent / "ai_engine"))

from ai_worker import AIWorker

async def test_simple_inference():
    """Test simple inference task"""
    print("🧪 Testing simple inference task...")
    
    config = {
        'models_cache_dir': './test_models_cache',
        'node_id': 'test_node'
    }
    
    worker = AIWorker(config)
    
    task_data = {
        'id': 1,
        'description': json.dumps({
            'model': 'distilbert-base-uncased',
            'input': 'This is a test sentence for sentiment analysis.',
            'task_type': 'classification',
            'parameters': {
                'return_all_scores': True
            }
        })
    }
    
    try:
        proof_hash, output = await worker.execute_task(task_data)
        
        print(f"✅ Task completed successfully!")
        print(f"Proof hash: {proof_hash}")
        print(f"Output preview: {output[:200]}...")
        
        # Parse and display result
        result = json.loads(output)
        print(f"\nExecution time: {result.get('execution_time', 'N/A'):.2f}s")
        print(f"Model used: {result.get('model', 'N/A')}")
        print(f"Task type: {result.get('task_type', 'N/A')}")
        
        return True
        
    except Exception as e:
        print(f"❌ Task failed: {e}")
        return False

async def test_text_generation():
    """Test text generation task"""
    print("\n🧪 Testing text generation task...")
    
    config = {
        'models_cache_dir': './test_models_cache',
        'node_id': 'test_node'
    }
    
    worker = AIWorker(config)
    
    task_data = {
        'id': 2,
        'description': json.dumps({
            'model': 'gpt2',
            'input': 'The future of artificial intelligence is',
            'task_type': 'text_generation',
            'parameters': {
                'max_length': 50,
                'temperature': 0.7
            }
        })
    }
    
    try:
        proof_hash, output = await worker.execute_task(task_data)
        
        print(f"✅ Text generation completed!")
        print(f"Proof hash: {proof_hash}")
        
        result = json.loads(output)
        print(f"Generated text: {result.get('result', 'N/A')}")
        
        return True
        
    except Exception as e:
        print(f"❌ Text generation failed: {e}")
        return False

async def test_embedding_generation():
    """Test embedding generation task"""
    print("\n🧪 Testing embedding generation task...")
    
    config = {
        'models_cache_dir': './test_models_cache',
        'node_id': 'test_node'
    }
    
    worker = AIWorker(config)
    
    task_data = {
        'id': 3,
        'description': json.dumps({
            'model': 'sentence-transformers/all-MiniLM-L6-v2',
            'input': 'This sentence will be converted to an embedding vector.',
            'task_type': 'embedding',
            'parameters': {}
        })
    }
    
    try:
        proof_hash, output = await worker.execute_task(task_data)
        
        print(f"✅ Embedding generation completed!")
        print(f"Proof hash: {proof_hash}")
        
        result = json.loads(output)
        embedding = result.get('result', [])
        print(f"Embedding dimension: {len(embedding) if isinstance(embedding, list) else 'N/A'}")
        
        return True
        
    except Exception as e:
        print(f"❌ Embedding generation failed: {e}")
        return False

async def main():
    """Run all test tasks"""
    print("🚀 DeAI AI Worker Test Suite")
    print("=" * 40)
    
    tests = [
        ("Simple Inference", test_simple_inference),
        ("Text Generation", test_text_generation),
        ("Embedding Generation", test_embedding_generation),
    ]
    
    results = []
    for test_name, test_func in tests:
        print(f"\n📋 Running: {test_name}")
        result = await test_func()
        results.append((test_name, result))
    
    print("\n" + "=" * 40)
    print("📊 Test Results Summary:")
    
    passed = 0
    for test_name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"  {test_name}: {status}")
        if result:
            passed += 1
    
    print(f"\nTotal: {passed}/{len(results)} tests passed")
    
    if passed == len(results):
        print("🎉 All tests passed! AI worker is functioning correctly.")
        return 0
    else:
        print("⚠️  Some tests failed. Check error messages above.")
        return 1

if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)