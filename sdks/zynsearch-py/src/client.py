import base64
from typing import Any, Dict, List, Optional, Union

# ==========================================
# 🤖 THE BASE ROUTER ENGINE
# ==========================================

class _BaseZynClient:
    """
    Internal foundation setup shared by both the Sync and Async walkie-talkies.
    This manages the endpoint cleanups, timeouts, and auth header calculations.
    """
    def __init__(
        self, 
        endpoint: string, 
        username: Optional[str] = None, 
        password: Optional[str] = None,
        timeout_seconds: float = 5.0
    ):
        # Clean up trailing slashes so URL additions don't break with double slashes
        self.endpoint = endpoint.rstrip("/")
        self.timeout = timeout_seconds
        
        # Build standard checklist of headers matching your Axum server rules
        self.headers = {
            "Accept": "application/json"
        }
        
        # If credentials are passed, convert them into an authorized Basic Base64 token
        if username and password:
            raw_credentials = f"{username}:{password}"
            # Encode strings to bytes, translate to base64, and decode back to a string
            encoded_bytes = base64.b64encode(raw_credentials.encode("utf-8"))
            self.headers["Authorization"] = f"Basic {encoded_bytes.decode('utf-8')}"

    def _build_index_payload(self, source_id: str, content: str, source_kind: Optional[str], replace_existing: Optional[bool]) -> Dict[str, Any]:
        """Translates Python camelCase/snake parameters into exactly what Axum expects."""
        payload = {
            "source_id": source_id,
            "content": content
        }
        if source_kind:
            payload["source_kind"] = source_kind
        if replace_existing is not None:
            payload["replace_existing"] = replace_existing
        return payload

    def _build_search_params(self, query: str, limit: Optional[int], explain: Optional[bool]) -> Dict[str, Any]:
        """Assembles URL query parameters for the /search endpoint."""
        params = {"q": query}
        if limit is not None:
            params["limit"] = limit
        if explain is not None:
            params["explain"] = str(explain).lower()
        return params


# ==========================================
# ⏱️ THE PATIENT CLIENT (SYNCHRONOUS)
# ==========================================

import httpx

class ZynSearchClient(_BaseZynClient):
    """
    The classic synchronous client. Blocks execution until the cluster replies.
    """
    def __init__(self, endpoint: str, username: Optional[str] = None, password: Optional[str] = None, timeout_seconds: float = 5.0):
        super().__init__(endpoint, username, password, timeout_seconds)
        # Spin up a reusable httpx connection pool
        self.client = httpx.Client(headers=self.headers, timeout=self.timeout)

    def close(self):
        """Closes the connection pool gracefully when done."""
        self.client.close()

    def index(self, source_id: str, content: str, source_kind: Optional[str] = None, replace_existing: Optional[bool] = None) -> Dict[str, Any]:
        """Pushes raw text data into a cluster shard."""
        url = f"{self.endpoint}/index"
        body = self._build_index_payload(source_id, content, source_kind, replace_existing)
        
        response = self.client.post(url, json=body)
        response.raise_for_status() # Automatically trigger exceptions for any API errors
        return response.json()

    def search(self, query: str, limit: Optional[int] = None, explain: Optional[bool] = None) -> Dict[str, Any]:
        """Executes a scatter-gather distributed cluster query."""
        url = f"{self.endpoint}/search"
        params = self._build_search_params(query, limit, explain)
        
        response = self.client.get(url, params=params)
        response.raise_for_status()
        return response.json()

    def delete(self, document_identity: Union[str, int]) -> Dict[str, Any]:
        """Removes a document entry from the index by its ID or source path."""
        url = f"{self.endpoint}/index/{document_identity}"
        response = self.client.delete(url)
        response.raise_for_status()
        return response.json()


# ==========================================
# ⚡ THE MULTI-TASKER CLIENT (ASYNCHRONOUS)
# ==========================================

class AsyncZynSearchClient(_BaseZynClient):
    """
    The high-throughput async client for non-blocking loop runtimes.
    """
    def __init__(self, endpoint: str, username: Optional[str] = None, password: Optional[str] = None, timeout_seconds: float = 5.0):
        super().__init__(endpoint, username, password, timeout_seconds)
        self.client = httpx.AsyncClient(headers=self.headers, timeout=self.timeout)

    async def close(self):
        """Asynchronously closes the network connections."""
        await self.client.aclose()

    async def index(self, source_id: str, content: str, source_kind: Optional[str] = None, replace_existing: Optional[bool] = None) -> Dict[str, Any]:
        """Asynchronously index a document payload into the engine cluster."""
        url = f"{self.endpoint}/index"
        body = self._build_index_payload(source_id, content, source_kind, replace_existing)
        
        response = await self.client.post(url, json=body)
        response.raise_for_status()
        return response.json()

    async def search(self, query: str, limit: Optional[int] = None, explain: Optional[bool] = None) -> Dict[str, Any]:
        """Asynchronously search across cluster nodes and return consolidated rankings."""
        url = f"{self.endpoint}/search"
        params = self._build_search_params(query, limit, explain)
        
        response = await self.client.get(url, params=params)
        response.raise_for_status()
        return response.json()

    async def delete(self, document_identity: Union[str, int]) -> Dict[str, Any]:
        """Asynchronously remove a record entry using the cluster deletion strategy."""
        url = f"{self.endpoint}/index/{document_identity}"
        response = await self.client.delete(url)
        response.raise_for_status()
        return response.json()