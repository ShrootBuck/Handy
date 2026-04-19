# Handy (Mistral fork)

This is my personal fork of [Handy](https://github.com/cjpais/Handy), a simple push-to-talk transcription app built with Tauri.

The only reason this fork exists is to swap out the transcription backend for **Mistral's Voxtral** models. A bunch of the original app's features (local Whisper models, post-processing LLMs, audio feedback, etc.) have been stripped out to keep it focused on doing one thing well: fast, high-quality cloud transcription with Mistral.

If you want the full-featured, actively maintained version, go use the [upstream project](https://github.com/cjpais/Handy). If you specifically want Mistral Voxtral transcription and nothing else, you're in the right place.
