import os
from dotenv import load_dotenv

load_dotenv()


class Config:
    BASE_URL = os.getenv('LLMUR_API_BASE_URL', 'http://0.0.0.0:8082')
    API_KEY = os.getenv('LLMUR_API_KEY', 'my-key-123')
    TIMEOUT = int(os.getenv('API_TIMEOUT', '30'))
    OPENAI_API_KEY = os.getenv('OPENAI_API_KEY')
    OPENAI_BASE_URL = os.getenv('OPENAI_BASE_URL', 'https://api.openai.com')
    OPENAI_MODEL = os.getenv('OPENAI_MODEL')
    OPENAI_CHAT_COMPLETIONS_MODEL = os.getenv('OPENAI_CHAT_COMPLETIONS_MODEL', OPENAI_MODEL)
    OPENAI_EMBEDDINGS_MODEL = os.getenv('OPENAI_EMBEDDINGS_MODEL', OPENAI_MODEL)

    AZURE_OPENAI_API_KEY = os.getenv('AZURE_OPENAI_API_KEY')
    AZURE_OPENAI_ENDPOINT = os.getenv('AZURE_OPENAI_ENDPOINT')
    AZURE_OPENAI_API_VERSION = os.getenv('AZURE_OPENAI_API_VERSION', 'v1')
    AZURE_OPENAI_DEPLOYMENT = os.getenv('AZURE_OPENAI_DEPLOYMENT')
    AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT = os.getenv('AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT', AZURE_OPENAI_DEPLOYMENT)
    AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT = os.getenv('AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT', AZURE_OPENAI_DEPLOYMENT)

    GEMINI_API_KEY = os.getenv('GEMINI_API_KEY')
    GEMINI_BASE_URL = os.getenv('GEMINI_BASE_URL', 'https://generativelanguage.googleapis.com')
    GEMINI_API_VERSION = os.getenv('GEMINI_API_VERSION', 'v1beta')
    GEMINI_MODEL = os.getenv('GEMINI_MODEL')
    GEMINI_CHAT_COMPLETIONS_MODEL = os.getenv('GEMINI_CHAT_COMPLETIONS_MODEL', GEMINI_MODEL)
    GEMINI_EMBEDDINGS_MODEL = os.getenv('GEMINI_EMBEDDINGS_MODEL', GEMINI_MODEL)

    @classmethod
    def get_headers(cls):
        return {
            'X-LLMur-Key': cls.API_KEY,
            'Content-Type': 'application/json'
        }
