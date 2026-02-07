#!/usr/bin/env python3
"""
Test script for Cloudflare Workers AI models via the MCP server.
Tests each model type to verify input/output formats.
"""

import json
import os
import requests
import subprocess
import sys
from typing import Dict, Any, List, Tuple
from dataclasses import dataclass, asdict
import time


def get_worker_url():
    """Get the worker URL from a CLI arg or wrangler."""
    if len(sys.argv) > 1:
        return sys.argv[1]
    try:
        result = subprocess.run(
            ["wrangler", "deployments", "list", "--json"],
            capture_output=True, text=True, timeout=10,
        )
        data = json.loads(result.stdout)
        if data and isinstance(data, list):
            for d in data:
                url = d.get("url", "")
                if url:
                    return url.rstrip("/") + "/mcp"
    except Exception:
        pass
    print("Usage: test_models.py <MCP_URL>", file=sys.stderr)
    print("  or run from a directory with wrangler.toml", file=sys.stderr)
    sys.exit(1)


MCP_URL = get_worker_url()
AUTH_TOKEN = os.environ.get("MCP_AUTH_TOKEN", "")

@dataclass
class TestResult:
    model_id: str
    model_type: str
    success: bool
    input_shape: Dict[str, Any]
    output_shape: Dict[str, Any]
    error: str = None
    neurons_used: int = None
    response_sample: str = None

class ModelTester:
    def __init__(self, url: str, auth_token: str):
        self.url = url
        self.headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {auth_token}"
        }
        self.request_id = 0
        self.results: List[TestResult] = []

    def call_mcp(self, method: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Make an MCP JSON-RPC call"""
        self.request_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        }

        response = requests.post(self.url, headers=self.headers, json=payload, timeout=60)
        response.raise_for_status()
        return response.json()

    def test_model(self, model_id: str, model_type: str, test_input: Dict[str, Any]) -> TestResult:
        """Test a single model"""
        print(f"Testing {model_id} ({model_type})...", end=" ")

        try:
            result = self.call_mcp("tools/call", {
                "name": model_id,
                "arguments": test_input
            })

            if "error" in result:
                print(f"❌ Error: {result['error']['message'][:100]}")
                return TestResult(
                    model_id=model_id,
                    model_type=model_type,
                    success=False,
                    input_shape={"provided": list(test_input.keys())},
                    output_shape={},
                    error=result['error']['message']
                )

            # Extract response
            content = result.get("result", {}).get("content", [])
            if content and len(content) > 0:
                text = content[0].get("text", "")

                # Try to parse the response JSON
                try:
                    # Response format is: JSON\n\n[Neurons used: N]
                    json_part = text.split("\n\n[Neurons used:")[0]
                    response_data = json.loads(json_part)

                    neurons_match = text.split("[Neurons used: ")
                    neurons = int(neurons_match[1].rstrip("]")) if len(neurons_match) > 1 else None

                    # Get sample of actual response
                    response_sample = None
                    if "response" in response_data:
                        resp = str(response_data["response"])
                        response_sample = resp[:100] if len(resp) > 100 else resp
                    elif "result" in response_data:
                        resp = str(response_data["result"])
                        response_sample = resp[:100] if len(resp) > 100 else resp

                    print(f"✅ ({neurons} neurons)")

                    return TestResult(
                        model_id=model_id,
                        model_type=model_type,
                        success=True,
                        input_shape={"provided": list(test_input.keys())},
                        output_shape={"keys": list(response_data.keys())},
                        neurons_used=neurons,
                        response_sample=response_sample
                    )
                except json.JSONDecodeError:
                    print(f"✅ (raw response)")
                    return TestResult(
                        model_id=model_id,
                        model_type=model_type,
                        success=True,
                        input_shape={"provided": list(test_input.keys())},
                        output_shape={"format": "raw_text"},
                        response_sample=text[:100]
                    )
            else:
                print("❌ No content in response")
                return TestResult(
                    model_id=model_id,
                    model_type=model_type,
                    success=False,
                    input_shape={"provided": list(test_input.keys())},
                    output_shape={},
                    error="No content in response"
                )

        except Exception as e:
            print(f"❌ Exception: {str(e)[:100]}")
            return TestResult(
                model_id=model_id,
                model_type=model_type,
                success=False,
                input_shape={"provided": list(test_input.keys())},
                output_shape={},
                error=str(e)
            )

    def run_tests(self):
        """Run tests for all model types"""

        # Text Generation Models
        text_gen_models = [
            ("@cf/meta/llama-3.1-8b-instruct", "text-generation"),
            ("@cf/meta/llama-3.1-70b-instruct", "text-generation"),
            ("@cf/meta/llama-3.2-1b-instruct", "text-generation"),
            ("@cf/mistral/mistral-7b-instruct-v0.1", "text-generation"),
            ("@cf/qwen/qwen2.5-coder-32b-instruct", "text-generation"),
        ]

        # Text Embeddings
        embedding_models = [
            ("@cf/baai/bge-base-en-v1.5", "text-embeddings"),
            ("@cf/baai/bge-large-en-v1.5", "text-embeddings"),
            ("@cf/baai/bge-m3", "text-embeddings"),
        ]

        # Text-to-Image
        image_gen_models = [
            ("@cf/stabilityai/stable-diffusion-xl-base-1.0", "text-to-image"),
            ("@cf/black-forest-labs/flux-1-schnell", "text-to-image"),
            ("@cf/bytedance/stable-diffusion-xl-lightning", "text-to-image"),
        ]

        # Speech Recognition
        speech_models = [
            ("@cf/openai/whisper", "speech-recognition"),
        ]

        # Image Classification/Object Detection
        vision_models = [
            ("@cf/microsoft/resnet-50", "image-classification"),
        ]

        # Test text generation
        print("\n=== Testing Text Generation Models ===")
        for model_id, model_type in text_gen_models:
            result = self.test_model(model_id, model_type, {
                "prompt": "What is the capital of France? Answer in one word."
            })
            self.results.append(result)
            time.sleep(0.5)  # Rate limiting

        # Test embeddings
        print("\n=== Testing Text Embedding Models ===")
        for model_id, model_type in embedding_models:
            result = self.test_model(model_id, model_type, {
                "text": "The quick brown fox jumps over the lazy dog"
            })
            self.results.append(result)
            time.sleep(0.5)

        # Test image generation
        print("\n=== Testing Text-to-Image Models ===")
        for model_id, model_type in image_gen_models:
            result = self.test_model(model_id, model_type, {
                "prompt": "A serene mountain landscape at sunset"
            })
            self.results.append(result)
            time.sleep(0.5)

        # Test speech recognition (needs audio data)
        print("\n=== Testing Speech Recognition Models ===")
        print("⚠️  Skipping Whisper - requires base64 audio data")
        # We'll skip this for now since we don't have sample audio

        # Test vision models (needs image data)
        print("\n=== Testing Vision Models ===")
        print("⚠️  Skipping vision models - require image data")

    def generate_report(self):
        """Generate a comprehensive report"""
        print("\n" + "="*80)
        print("MODEL TESTING REPORT")
        print("="*80)

        # Group by model type
        by_type: Dict[str, List[TestResult]] = {}
        for result in self.results:
            if result.model_type not in by_type:
                by_type[result.model_type] = []
            by_type[result.model_type].append(result)

        # Report by type
        for model_type, results in sorted(by_type.items()):
            print(f"\n{'─'*80}")
            print(f"Model Type: {model_type.upper()}")
            print(f"{'─'*80}")

            successful = [r for r in results if r.success]
            failed = [r for r in results if not r.success]

            print(f"Success Rate: {len(successful)}/{len(results)}")

            if successful:
                print(f"\n✅ Successful Tests:")
                for result in successful:
                    print(f"\n  Model: {result.model_id}")
                    print(f"  Input Shape: {result.input_shape}")
                    print(f"  Output Shape: {result.output_shape}")
                    if result.neurons_used:
                        print(f"  Neurons Used: {result.neurons_used}")
                    if result.response_sample:
                        print(f"  Sample Response: {result.response_sample}")

            if failed:
                print(f"\n❌ Failed Tests:")
                for result in failed:
                    print(f"\n  Model: {result.model_id}")
                    print(f"  Input Shape: {result.input_shape}")
                    print(f"  Error: {result.error[:200] if result.error else 'Unknown'}")

        # Summary statistics
        print(f"\n{'='*80}")
        print("SUMMARY")
        print(f"{'='*80}")
        total = len(self.results)
        successful = len([r for r in self.results if r.success])
        print(f"Total Tests: {total}")
        print(f"Successful: {successful}")
        print(f"Failed: {total - successful}")
        print(f"Success Rate: {successful/total*100:.1f}%")

        # Neuron usage
        total_neurons = sum(r.neurons_used for r in self.results if r.neurons_used)
        print(f"\nTotal Neurons Used: {total_neurons}")

        # Input/Output format summary
        print(f"\n{'─'*80}")
        print("INPUT/OUTPUT FORMAT SUMMARY BY MODEL TYPE")
        print(f"{'─'*80}")

        for model_type in sorted(by_type.keys()):
            successful = [r for r in by_type[model_type] if r.success]
            if successful:
                print(f"\n{model_type.upper()}:")
                # Get unique input shapes
                input_keys = set()
                output_keys = set()
                for r in successful:
                    if isinstance(r.input_shape.get("provided"), list):
                        input_keys.update(r.input_shape["provided"])
                    if isinstance(r.output_shape.get("keys"), list):
                        output_keys.update(r.output_shape["keys"])

                print(f"  Input Fields: {sorted(input_keys) if input_keys else 'N/A'}")
                print(f"  Output Fields: {sorted(output_keys) if output_keys else 'raw/varies'}")

def main():
    """Main entry point"""
    print("Cloudflare Workers AI Model Testing Script")
    print("=" * 80)

    tester = ModelTester(MCP_URL, AUTH_TOKEN)
    tester.run_tests()
    tester.generate_report()

    return 0 if all(r.success for r in tester.results) else 1

if __name__ == "__main__":
    sys.exit(main())
