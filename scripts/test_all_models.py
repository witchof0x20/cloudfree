#!/usr/bin/env python3
"""
Comprehensive test script for ALL Cloudflare Workers AI models.
Tests every available model to verify input/output formats.
"""

import json
import os
import requests
import subprocess
import sys
from typing import Dict, Any, List
from dataclasses import dataclass
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
            # Extract from deployment URL
            for d in data:
                url = d.get("url", "")
                if url:
                    return url.rstrip("/") + "/mcp"
    except Exception:
        pass
    print("Usage: test_all_models.py <MCP_URL>", file=sys.stderr)
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

# Complete model catalog with full IDs including organization paths
ALL_MODELS = {
    "text-generation": [
        "@cf/openai/gpt-oss-120b", "@cf/openai/gpt-oss-20b",
        "@cf/meta/llama-4-scout-17b-16e-instruct", "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
        "@cf/meta/llama-3.1-8b-instruct-fast", "@cf/ibm/granite-4.0-h-micro",
        "@cf/aisingapore/gemma-sea-lion-v4-27b-it", "@cf/qwen/qwen3-30b-a3b-fp8",
        "@cf/google/gemma-3-12b-it", "@cf/mistralai/mistral-small-3.1-24b-instruct",
        "@cf/qwen/qwq-32b", "@cf/qwen/qwen2.5-coder-32b-instruct",
        "@cf/meta/llama-guard-3-8b", "@cf/deepseek/deepseek-r1-distill-qwen-32b",
        "@cf/meta/llama-3.2-1b-instruct", "@cf/meta/llama-3.2-3b-instruct",
        "@cf/meta/llama-3.2-11b-vision-instruct", "@cf/meta/llama-3.1-8b-instruct-awq",
        "@cf/meta/llama-3.1-8b-instruct-fp8", "@cf/meta/llama-3.1-8b-instruct",
        "@cf/meta/llama-3.1-70b-instruct", "@cf/meta/meta-llama-3-8b-instruct",
        "@cf/meta/llama-3-8b-instruct-awq", "@cf/meta/llama-3-8b-instruct",
        "@cf/meta/llama-2-7b-chat-fp16", "@cf/meta/llama-2-7b-chat-int8",
        "@cf/mistral/mistral-7b-instruct-v0.1", "@cf/mistralai/mistral-7b-instruct-v0.2",
        "@cf/mistralai/mistral-7b-instruct-v0.1-awq", "@cf/google/gemma-7b-it",
        "@cf/nousresearch/hermes-2-pro-mistral-7b", "@cf/microsoft/phi-2",
        "@cf/fblgit/una-cybertron-7b-v2-bf16"
    ],
    "text-to-image": [
        "@cf/black-forest-labs/flux-1-schnell", "@cf/bytedance/stable-diffusion-xl-lightning",
        "@cf/lykon/dreamshaper-8-lcm", "@cf/runwayml/stable-diffusion-v1-5-img2img",
        "@cf/runwayml/stable-diffusion-v1-5-inpainting",
        "@cf/stabilityai/stable-diffusion-xl-base-1.0"
    ],
    "text-to-speech": [
        "@cf/deepgram/aura-2-es", "@cf/deepgram/aura-2-en",
        "@cf/deepgram/aura-1"
    ],
    "speech-recognition": [
        "@cf/deepgram/flux", "@cf/deepgram/nova-3",
        "@cf/openai/whisper-large-v3-turbo", "@cf/openai/whisper-tiny-en",
        "@cf/openai/whisper"
    ],
    "text-embeddings": [
        "@cf/pfnet/plamo-embedding-1b", "@cf/google/embeddinggemma-300m",
        "@cf/qwen/qwen3-embedding-0.6b", "@cf/baai/bge-m3",
        "@cf/baai/bge-large-en-v1.5", "@cf/baai/bge-small-en-v1.5",
        "@cf/baai/bge-base-en-v1.5"
    ],
    "text-classification": [
        "@cf/baai/bge-reranker-base", "@cf/huggingface/distilbert-sst-2-int8"
    ],
    "translation": [
        "@cf/ai4bharat/indictrans2-en-indic-1B", "@cf/meta/m2m100-1.2b"
    ],
    "image-to-text": [
        "@cf/llava-hf/llava-1.5-7b-hf", "@cf/unum/uform-gen2-qwen-500m"
    ],
    "summarization": [
        "@cf/facebook/bart-large-cnn"
    ],
    "object-detection": [
        "@cf/facebook/detr-resnet-50"
    ],
    "image-classification": [
        "@cf/microsoft/resnet-50"
    ]
}

# Test inputs for each category
TEST_INPUTS = {
    "text-generation": {"prompt": "What is 2+2? Answer in one word."},
    "text-to-image": {"prompt": "A serene mountain landscape"},
    "text-to-speech": {"text": "Hello world"},
    "speech-recognition": None,  # Needs audio data - skip
    "text-embeddings": {"text": "The quick brown fox jumps over the lazy dog"},
    "text-classification": {"text": "I love this product, it's amazing!"},
    "translation": {"text": "Hello, how are you?", "source_lang": "en", "target_lang": "es"},
    "image-to-text": None,  # Needs image data - skip
    "summarization": {"text": "The Industrial Revolution was a period of major industrialization and innovation that took place during the late 1700s and early 1800s."},
    "object-detection": None,  # Needs image data - skip
    "image-classification": None,  # Needs image data - skip
}

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

        response = requests.post(self.url, headers=self.headers, json=payload, timeout=120)
        response.raise_for_status()
        return response.json()

    def test_model(self, model_id: str, model_type: str, test_input: Dict[str, Any]) -> TestResult:
        """Test a single model"""
        print(f"  {model_id}...", end=" ", flush=True)

        try:
            result = self.call_mcp("tools/call", {
                "name": model_id,  # Already includes @cf/ prefix
                "arguments": test_input
            })

            if "error" in result:
                error_msg = result['error']['message']
                # Shorten long errors
                if len(error_msg) > 100:
                    error_msg = error_msg[:97] + "..."
                print(f"❌ {error_msg}")
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

                try:
                    # Response format: JSON\n\n[Neurons used: N]
                    json_part = text.split("\n\n[Neurons used:")[0]
                    response_data = json.loads(json_part)

                    neurons_match = text.split("[Neurons used: ")
                    neurons = int(neurons_match[1].rstrip("]")) if len(neurons_match) > 1 else None

                    # Get sample response
                    response_sample = None
                    if "response" in response_data:
                        resp = str(response_data["response"])
                        response_sample = resp[:50]
                    elif "result" in response_data:
                        resp = str(response_data["result"])
                        response_sample = resp[:50]

                    print(f"✅ ({neurons}n)")

                    return TestResult(
                        model_id=model_id,
                        model_type=model_type,
                        success=True,
                        input_shape={"provided": list(test_input.keys())},
                        output_shape={"keys": list(response_data.keys())},
                        neurons_used=neurons,
                        response_sample=response_sample
                    )
                except (json.JSONDecodeError, ValueError):
                    print(f"✅ (raw)")
                    return TestResult(
                        model_id=model_id,
                        model_type=model_type,
                        success=True,
                        input_shape={"provided": list(test_input.keys())},
                        output_shape={"format": "raw"},
                        response_sample=text[:50]
                    )
            else:
                print("❌ No content")
                return TestResult(
                    model_id=model_id,
                    model_type=model_type,
                    success=False,
                    input_shape={"provided": list(test_input.keys())},
                    output_shape={},
                    error="No content in response"
                )

        except Exception as e:
            error_str = str(e)
            if len(error_str) > 100:
                error_str = error_str[:97] + "..."
            print(f"❌ {error_str}")
            return TestResult(
                model_id=model_id,
                model_type=model_type,
                success=False,
                input_shape={"provided": list(test_input.keys()) if test_input else []},
                output_shape={},
                error=str(e)
            )

    def run_all_tests(self):
        """Test all models"""
        total_models = sum(len(models) for models in ALL_MODELS.values())
        print(f"\n{'='*80}")
        print(f"Testing {total_models} models across {len(ALL_MODELS)} categories")
        print(f"{'='*80}\n")

        for category, models in ALL_MODELS.items():
            test_input = TEST_INPUTS.get(category)

            if test_input is None:
                print(f"{category.upper()} ({len(models)} models): ⚠️  SKIPPED (requires special input)")
                continue

            print(f"{category.upper()} ({len(models)} models):")

            for model_id in models:
                result = self.test_model(model_id, category, test_input)
                self.results.append(result)
                time.sleep(0.2)  # Rate limiting

            print()  # Blank line between categories

    def generate_report(self):
        """Generate comprehensive report"""
        print(f"\n{'='*80}")
        print("COMPREHENSIVE MODEL TEST REPORT")
        print(f"{'='*80}\n")

        # Overall statistics
        total = len(self.results)
        successful = len([r for r in self.results if r.success])
        failed = total - successful

        print(f"Total Models Tested: {total}")
        print(f"Successful: {successful} ({successful/total*100:.1f}%)")
        print(f"Failed: {failed} ({failed/total*100:.1f}%)")

        if any(r.neurons_used for r in self.results):
            total_neurons = sum(r.neurons_used or 0 for r in self.results)
            print(f"Total Neurons Used: {total_neurons:,}")

        # Group by category
        by_category: Dict[str, List[TestResult]] = {}
        for result in self.results:
            if result.model_type not in by_category:
                by_category[result.model_type] = []
            by_category[result.model_type].append(result)

        # Report by category
        for category in sorted(by_category.keys()):
            results = by_category[category]
            successful = [r for r in results if r.success]
            failed = [r for r in results if not r.success]

            print(f"\n{'-'*80}")
            print(f"{category.upper()}")
            print(f"{'-'*80}")
            print(f"Success Rate: {len(successful)}/{len(results)} ({len(successful)/len(results)*100:.0f}%)")

            if successful and len(successful) <= 10:
                print(f"\n✅ Successful Models:")
                for r in successful:
                    print(f"  • {r.model_id.split('/')[-1]}")

            if failed:
                print(f"\n❌ Failed Models ({len(failed)}):")
                # Group failures by error type
                error_groups: Dict[str, List[str]] = {}
                for r in failed:
                    error_key = r.error[:50] if r.error else "Unknown"
                    if error_key not in error_groups:
                        error_groups[error_key] = []
                    error_groups[error_key].append(r.model_id.split('/')[-1])

                for error, model_list in error_groups.items():
                    print(f"\n  Error: {error}")
                    for model in model_list[:5]:  # Show first 5
                        print(f"    • {model}")
                    if len(model_list) > 5:
                        print(f"    ... and {len(model_list) - 5} more")

            # Show unique output formats for successful tests
            if successful:
                output_formats = set()
                for r in successful:
                    if "keys" in r.output_shape:
                        output_formats.add(tuple(sorted(r.output_shape["keys"])))

                if len(output_formats) > 0:
                    print(f"\n📊 Output Formats:")
                    for fmt in output_formats:
                        print(f"  {list(fmt)}")

def main():
    """Main entry point"""
    tester = ModelTester(MCP_URL, AUTH_TOKEN)
    tester.run_all_tests()
    tester.generate_report()

    # Return 0 if at least 50% succeeded
    success_rate = len([r for r in tester.results if r.success]) / len(tester.results) if tester.results else 0
    return 0 if success_rate >= 0.5 else 1

if __name__ == "__main__":
    sys.exit(main())
