# cloudfree
Free AI models in Claude Code via Cloudflare Workers AI + MCP.


NOTE: this is super simple, vibe coded, and nowhere near feature complete or necessarily correct. It's also extremely messy

## Deploy

```sh
wrangler deploy
```

## Set up auth

Generate a secret and add it:

```sh
openssl rand -hex 32
wrangler secret put MCP_AUTH_TOKEN
```

## Add to Claude Code

```sh
claude mcp add --scope user --transport http cloudfree https://cloudfree-mcp.yourworkspacegoeshere.workers.dev/mcp -H "Authorization: Bearer secret_goes_here"
```

## Models

LLMs: Llama 3.1 8B, Llama 3.1 70B, Llama 3.2 1B, Mistral 7B, Qwen 2.5 Coder 32B
Embeddings: BGE Base/Large English v1.5, BGE M3
Image: Stable Diffusion XL, SDXL Lightning, Flux 1 Schnell
Audio: Whisper

Daily limit: 10,000 neurons (Cloudflare free tier).
