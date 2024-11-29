from SecureStorage.secure_store import SecureStore
from typing import Optional


class SecretsServer:

    def __init__(self):
        self._store = SecureStore()

    def get_request(self, key: str) -> Optional[str]:

        return self._store.get_secret(key)

    def store_request(self, key: str, value: str) -> None:
        self._store.store_secret(key, value)
