# CloudFree MCP Server - TODO

## Known Issues & Limitations

### 🔴 High Priority

#### 1. Speech Recognition Models - Audio Input Support
**Status:** Not implemented
**Models Affected:**
- `@cf/deepgram/flux`
- `@cf/deepgram/nova-3`
- `@cf/openai/whisper-large-v3-turbo`
- `@cf/openai/whisper-tiny-en`
- `@cf/openai/whisper`

**Issue:** These models require base64-encoded audio data as input. Current implementation doesn't support this format.

**Required Input Format:**
```json
{
  "audio": "base64_encoded_audio_data",
  "language": "en" // optional
}
```

**Action Items:**
- [ ] Add audio file encoding support
- [ ] Update input schema for audio models
- [ ] Add audio format validation
- [ ] Test with sample audio files

---

#### 2. Image-to-Text Models - Image Input Support
**Status:** Not implemented
**Models Affected:**
- `@cf/llava-hf/llava-1.5-7b-hf`
- `@cf/unum/uform-gen2-qwen-500m`

**Issue:** These models require image data as input (base64 or URL).

**Required Input Format:**
```json
{
  "image": "base64_encoded_image_data" // or image URL
}
```

**Action Items:**
- [ ] Add image encoding support
- [ ] Support both base64 and URL inputs
- [ ] Update input schema for vision models
- [ ] Test with sample images

---

#### 3. Object Detection Models - Image Input Support
**Status:** Not implemented
**Models Affected:**
- `@cf/facebook/detr-resnet-50`

**Issue:** Requires image input for object detection.

**Required Input Format:**
```json
{
  "image": "base64_encoded_image_data"
}
```

**Action Items:**
- [ ] Add image input support
- [ ] Parse object detection output format
- [ ] Add bounding box visualization helpers

---

#### 4. Image Classification Models - Image Input Support
**Status:** Not implemented
**Models Affected:**
- `@cf/microsoft/resnet-50`

**Issue:** Requires image input for classification.

**Required Input Format:**
```json
{
  "image": "base64_encoded_image_data"
}
```

**Action Items:**
- [ ] Add image input support
- [ ] Parse classification output (labels + scores)
- [ ] Test with sample images

---

### 🟡 Medium Priority

#### 5. Text Classification Models - Incorrect Input Format
**Status:** Failing (0/2 models working)
**Models Affected:**
- `@cf/baai/bge-reranker-base`
- `@cf/huggingface/distilbert-sst-2-int8` (partial failure)

**Issue:** These models need specific input formats, not just `{"text": "..."}`.

**Expected Input Formats:**

For reranker:
```json
{
  "query": "string",
  "documents": ["string1", "string2", ...]
}
```

For sentiment classification:
```json
{
  "text": "string"  // might work, but output parsing fails
}
```

**Action Items:**
- [ ] Research correct input format for bge-reranker-base
- [ ] Fix output parsing for distilbert-sst-2-int8
- [ ] Update model registry with correct schemas
- [ ] Add tests for classification models

---

#### 6. Summarization Models - Input Format Issues
**Status:** Failing (0/1 models working)
**Models Affected:**
- `@cf/facebook/bart-large-cnn`

**Issue:** Model returns "required properties" error - input format not correct.

**Action Items:**
- [ ] Research correct input format for BART
- [ ] Update input schema in model registry
- [ ] Test with sample long text
- [ ] Document expected output format

---

#### 7. Image-to-Image Models - Special Input Requirements
**Status:** Failing (0/2 models working)
**Models Affected:**
- `@cf/runwayml/stable-diffusion-v1-5-img2img`
- `@cf/runwayml/stable-diffusion-v1-5-inpainting`

**Issue:** These models need both a prompt AND a source image. Inpainting also needs a mask.

**Required Input Format:**

For img2img:
```json
{
  "prompt": "string",
  "image": "base64_encoded_image_data",
  "strength": 0.8  // optional
}
```

For inpainting:
```json
{
  "prompt": "string",
  "image": "base64_encoded_image_data",
  "mask": "base64_encoded_mask_data"
}
```

**Action Items:**
- [ ] Add multi-input support (prompt + image)
- [ ] Add mask input support for inpainting
- [ ] Update model schemas
- [ ] Test with sample images

---

### 🟢 Low Priority / Informational

#### 8. Deprecated Models
**Models:**
- `@cf/fblgit/una-cybertron-7b-v2-bf16`

**Action:** Remove from model registry or mark as deprecated

---

#### 9. Non-Existent Models
**Models Failing with "No such model" Error:**
- `@cf/ibm/granite-4.0-h-micro`
- `@cf/deepseek/deepseek-r1-distill-qwen-32b`
- `@cf/meta/meta-llama-3-8b-instruct`
- `@cf/mistralai/mistral-7b-instruct-v0.2`
- `@cf/mistralai/mistral-7b-instruct-v0.1-awq`
- `@cf/google/gemma-7b-it`
- `@cf/nousresearch/hermes-2-pro-mistral-7b`

**Note:** These may be:
- Behind a waitlist (require opt-in)
- Temporarily unavailable
- Renamed/moved to different IDs
- Deprecated without notice

**Action Items:**
- [ ] Verify current availability on Cloudflare dashboard
- [ ] Remove non-existent models from registry
- [ ] Check for renamed model IDs
- [ ] Document opt-in requirements if applicable

---

#### 10. Special Requirements Models
**Models Requiring Opt-In:**
- `@cf/meta/llama-3.2-11b-vision-instruct` - Error: "Prior to using this model, you must..."

**Action:** Document opt-in process in README

---

#### 11. Capacity-Limited Models
**Models Temporarily at Capacity:**
- `@cf/ai4bharat/indictrans2-en-indic-1B`

**Note:** These are temporarily overloaded. Implement retry logic with backoff.

**Action Items:**
- [ ] Add automatic retry with exponential backoff
- [ ] Surface capacity errors to user gracefully
- [ ] Document rate limiting behavior

---

## Working Model Count Summary

| Category | Working | Total | Success Rate |
|----------|---------|-------|--------------|
| Text Generation | 23 | 33 | 70% |
| Text Embeddings | 7 | 7 | **100%** ✅ |
| Text-to-Image | 4 | 6 | 67% |
| Text-to-Speech | 3 | 3 | **100%** ✅ |
| Translation | 1 | 2 | 50% |
| Text Classification | 0 | 2 | 0% ❌ |
| Summarization | 0 | 1 | 0% ❌ |
| Speech Recognition | 0 | 5 | Not Tested |
| Image-to-Text | 0 | 2 | Not Tested |
| Object Detection | 0 | 1 | Not Tested |
| Image Classification | 0 | 1 | Not Tested |
| **TOTAL** | **38** | **54** | **70.4%** |

---

## Future Enhancements

### Features to Add
- [ ] **Streaming Support** - Real-time token streaming for LLMs
- [ ] **Batch Processing** - Process multiple requests efficiently
- [ ] **Usage Analytics** - Track neuron usage by model/category
- [ ] **Model Fallbacks** - Auto-fallback to alternative models on failure
- [ ] **Response Caching** - Cache identical requests to save neurons
- [ ] **Rate Limiting** - Client-side rate limiting to avoid 429 errors
- [ ] **File Upload Support** - Direct file upload for images/audio
- [ ] **Model Recommendations** - Suggest best model for user's task
- [ ] **Cost Estimator** - Preview neuron cost before execution

### Documentation Improvements
- [ ] Add examples for each model category
- [ ] Document neuron cost optimization strategies
- [ ] Create troubleshooting guide
- [ ] Add API usage examples for each model type
- [ ] Document best practices for staying within free tier

---

## Testing Improvements

- [ ] Add automated daily model tests
- [ ] Test with various input sizes to understand token limits
- [ ] Create regression test suite
- [ ] Add performance benchmarks
- [ ] Test concurrent request handling
- [ ] Validate neuron tracking accuracy

---

**Last Updated:** 2026-02-07
**Test Results Based On:** Comprehensive test of 54 models
