from SecureStorage.secure_store import SecureStore
from typing import Optional


class SecretsServer:

    def __init__(self):
        self._store = SecureStore()
        try:
            self._store.store_secret("DB_PASSWORD", "super_secret_123")
            self._store.store_secret("API_KEY", "very_secret_key_456")
        except Exception as e:
            print(f"Error initializing secrets: {e}")
            raise

    def get_request(self, key: str) -> Optional[str]:

        return self._store.get_secret(key)

    def store_request(self, key: str, value: str) -> None:
        self._store.store_secret(key, value)
