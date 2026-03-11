import json
import time

import pytest

def _jsonl(lines):
    return "\n".join(json.dumps(line) for line in lines) + "\n"


def _chat_line(custom_id, deployment_name, content="Hello"):
    return {
        "custom_id": custom_id,
        "method": "POST",
        "url": "/v1/chat/completions",
        "body": {
            "model": deployment_name,
            "messages": [
                {"role": "user", "content": content},
            ],
            "max_tokens": 8,
        },
    }


def _embeddings_line(custom_id, deployment_name, content="Hello"):
    return {
        "custom_id": custom_id,
        "method": "POST",
        "url": "/v1/embeddings",
        "body": {
            "model": deployment_name,
            "input": content,
        },
    }


def _assert_batch_object(batch, endpoint):
    assert "id" in batch
    assert batch["object"] == "batch"
    assert batch["endpoint"] == endpoint
    assert batch["completion_window"] == "24h"
    assert batch["status"] in {
        "validating",
        "in_progress",
        "finalizing",
        "completed",
        "failed",
        "expired",
        "cancelling",
        "cancelled",
    }


def _assert_lifecycle_surface_contract(payload):
    counts = payload.get("request_counts")
    status = payload["status"]
    if status in {"validating", "in_progress", "finalizing", "cancelling"}:
        assert payload.get("error_file_id") is None
        assert payload.get("output_file_id") is None
    if status == "validating" and counts is not None:
        assert counts["completed"] == 0
        assert counts["failed"] == 0
    if status == "completed" and counts is not None and counts["failed"] == 0:
        assert payload.get("error_file_id") is None
    if status == "cancelled" and counts is not None and counts["failed"] == 0:
        assert payload.get("error_file_id") is None


class TestBatchFiles:
    def test_batch_file_openai_chat(self, api_client, openai_chat_provider_setup):
        content = _jsonl([
            _chat_line("openai-1", openai_chat_provider_setup["deployment_name"]),
        ])

        response = api_client.create_file(
            openai_chat_provider_setup["virtual_key"],
            "openai-chat-batch.jsonl",
            content,
            "batch",
        )

        assert response.status_code == 200
        payload = response.json()
        assert payload["purpose"] == "batch"

        file_id = payload["id"]
        get_resp = api_client.get_file(file_id, openai_chat_provider_setup["virtual_key"])
        assert get_resp.status_code == 200

        list_resp = api_client.list_files(
            openai_chat_provider_setup["virtual_key"],
            params={"purpose": "batch"},
        )
        assert list_resp.status_code == 200
        ids = [item["id"] for item in list_resp.json().get("data", [])]
        assert file_id in ids

        delete_resp = api_client.delete_file(file_id, openai_chat_provider_setup["virtual_key"])
        assert delete_resp.status_code == 200

    def test_batch_file_azure_chat(self, api_client, azure_chat_batch_provider_setup):
        content = _jsonl([
            _chat_line("azure-1", azure_chat_batch_provider_setup["deployment_name"]),
        ])

        response = api_client.create_file(
            azure_chat_batch_provider_setup["virtual_key"],
            "azure-chat-batch.jsonl",
            content,
            "batch",
        )

        assert response.status_code == 200
        payload = response.json()
        assert payload["purpose"] == "batch"

        file_id = payload["id"]
        delete_resp = api_client.delete_file(file_id, azure_chat_batch_provider_setup["virtual_key"])
        assert delete_resp.status_code == 200

    def test_batch_file_azure_embeddings(self, api_client, azure_embeddings_batch_provider_setup):
        content = _jsonl([
            _embeddings_line("azure-emb-1", azure_embeddings_batch_provider_setup["deployment_name"]),
        ])

        response = api_client.create_file(
            azure_embeddings_batch_provider_setup["virtual_key"],
            "azure-emb-batch.jsonl",
            content,
            "batch",
        )

        assert response.status_code == 200
        payload = response.json()
        assert payload["purpose"] == "batch"

        file_id = payload["id"]
        delete_resp = api_client.delete_file(file_id, azure_embeddings_batch_provider_setup["virtual_key"])
        assert delete_resp.status_code == 200

    def test_batch_file_gemini_chat(self, api_client, gemini_chat_provider_setup):
        content = _jsonl([
            _chat_line("gemini-1", gemini_chat_provider_setup["deployment_name"]),
        ])

        response = api_client.create_file(
            gemini_chat_provider_setup["virtual_key"],
            "gemini-chat-batch.jsonl",
            content,
            "batch",
        )

        assert response.status_code == 200
        payload = response.json()
        assert payload["purpose"] == "batch"

        file_id = payload["id"]
        delete_resp = api_client.delete_file(file_id, gemini_chat_provider_setup["virtual_key"])
        assert delete_resp.status_code == 200

    def test_batch_file_gemini_embeddings_unsupported(self, api_client, gemini_embeddings_provider_setup):
        content = _jsonl([
            _embeddings_line("gemini-emb-1", gemini_embeddings_provider_setup["deployment_name"]),
        ])

        response = api_client.create_file(
            gemini_embeddings_provider_setup["virtual_key"],
            "gemini-emb-batch.jsonl",
            content,
            "batch",
        )

        assert response.status_code == 400


class TestBatches:
    def test_batch_create_openai_chat(self, api_client, openai_chat_provider_setup):
        content = _jsonl([
            _chat_line("openai-batch-1", openai_chat_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            openai_chat_provider_setup["virtual_key"],
            "openai-chat-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, openai_chat_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

        get_resp = api_client.get_batch(batch["id"], openai_chat_provider_setup["virtual_key"])
        assert get_resp.status_code == 200

        list_resp = api_client.list_batches(openai_chat_provider_setup["virtual_key"])
        assert list_resp.status_code == 200
        ids = [item["id"] for item in list_resp.json().get("data", [])]
        assert batch["id"] in ids

    def test_batch_create_azure_chat(self, api_client, azure_chat_batch_provider_setup):
        content = _jsonl([
            _chat_line("azure-batch-1", azure_chat_batch_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            azure_chat_batch_provider_setup["virtual_key"],
            "azure-chat-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, azure_chat_batch_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

    def test_batch_create_azure_embeddings(self, api_client, azure_embeddings_batch_provider_setup):
        content = _jsonl([
            _embeddings_line("azure-batch-emb-1", azure_embeddings_batch_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            azure_embeddings_batch_provider_setup["virtual_key"],
            "azure-emb-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/embeddings",
            "completion_window": "24h",
        }, azure_embeddings_batch_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/embeddings")

    def test_batch_create_gemini_chat(self, api_client, gemini_chat_provider_setup):
        content = _jsonl([
            _chat_line("gemini-batch-1", gemini_chat_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            gemini_chat_provider_setup["virtual_key"],
            "gemini-chat-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, gemini_chat_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

    def test_batch_retrieve_gemini_polling_does_not_fail(self, api_client, gemini_chat_provider_setup):
        content = _jsonl([
            _chat_line("gemini-batch-retrieve-1", gemini_chat_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            gemini_chat_provider_setup["virtual_key"],
            "gemini-chat-batch-retrieve.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, gemini_chat_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

        final_payload = None
        for _ in range(40):
            get_resp = api_client.get_batch(batch["id"], gemini_chat_provider_setup["virtual_key"])
            assert get_resp.status_code == 200, get_resp.text

            payload = get_resp.json()
            final_payload = payload
            _assert_batch_object(payload, "/v1/chat/completions")
            _assert_lifecycle_surface_contract(payload)
            counts = payload.get("request_counts")
            if payload["status"] in {"failed", "expired", "cancelled"}:
                break
            if payload["status"] == "completed" and payload.get("output_file_id"):
                if payload["status"] == "completed":
                    assert counts is not None
                    assert counts["completed"] + counts["failed"] == counts["total"]
                break
            time.sleep(1)

        assert final_payload is not None
        if final_payload["status"] == "completed":
            assert final_payload.get("output_file_id") is not None

    def test_batch_create_multi_provider_chat(self, api_client, multi_provider_chat_batch_setup):
        lines = []
        for provider in multi_provider_chat_batch_setup["providers"]:
            lines.append(_chat_line(f"multi-{provider['label']}", provider["deployment_name"]))
        content = _jsonl(lines)
        file_resp = api_client.create_file(
            multi_provider_chat_batch_setup["virtual_key"],
            "multi-provider-chat-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, multi_provider_chat_batch_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

    def test_batch_retrieve_multi_provider_counts_consistent(self, api_client, multi_provider_chat_batch_setup):
        lines = []
        for provider in multi_provider_chat_batch_setup["providers"]:
            lines.append(_chat_line(f"multi-retrieve-{provider['label']}", provider["deployment_name"]))

        content = _jsonl(lines)
        file_resp = api_client.create_file(
            multi_provider_chat_batch_setup["virtual_key"],
            "multi-provider-chat-retrieve-batch.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, multi_provider_chat_batch_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

        expected_total = len(lines)
        final_payload = None
        for _ in range(60):
            get_resp = api_client.get_batch(batch["id"], multi_provider_chat_batch_setup["virtual_key"])
            assert get_resp.status_code == 200, get_resp.text
            payload = get_resp.json()
            final_payload = payload
            _assert_batch_object(payload, "/v1/chat/completions")
            assert payload["id"] == batch["id"]
            _assert_lifecycle_surface_contract(payload)

            counts = payload.get("request_counts")
            if counts is not None:
                assert counts["total"] == expected_total

            if payload["status"] in {"failed", "expired", "cancelled"}:
                break
            if payload["status"] == "completed" and payload.get("output_file_id"):
                assert counts is not None
                if payload["status"] == "completed":
                    assert counts["completed"] + counts["failed"] == counts["total"]
                break
            time.sleep(1)

        assert final_payload is not None
        if final_payload["status"] == "completed":
            assert final_payload.get("output_file_id") is not None

    def test_batch_list_lifecycle_surface_contract(self, api_client, gemini_chat_provider_setup):
        content = _jsonl([
            _chat_line("gemini-list-contract-1", gemini_chat_provider_setup["deployment_name"]),
            _chat_line("gemini-list-contract-2", gemini_chat_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            gemini_chat_provider_setup["virtual_key"],
            "gemini-chat-batch-list-contract.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, gemini_chat_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

        found = None
        for _ in range(20):
            list_resp = api_client.list_batches(gemini_chat_provider_setup["virtual_key"])
            assert list_resp.status_code == 200
            payload = list_resp.json()
            for item in payload.get("data", []):
                if item.get("id") == batch["id"]:
                    found = item
                    break
            if found is not None:
                break
            time.sleep(1)

        assert found is not None
        _assert_batch_object(found, "/v1/chat/completions")
        _assert_lifecycle_surface_contract(found)

    def test_batch_cancel_lifecycle_surface_contract(self, api_client, azure_chat_batch_provider_setup):
        content = _jsonl([
            _chat_line("azure-cancel-contract-1", azure_chat_batch_provider_setup["deployment_name"]),
            _chat_line("azure-cancel-contract-2", azure_chat_batch_provider_setup["deployment_name"]),
        ])
        file_resp = api_client.create_file(
            azure_chat_batch_provider_setup["virtual_key"],
            "azure-chat-batch-cancel-contract.jsonl",
            content,
            "batch",
        )
        assert file_resp.status_code == 200
        file_id = file_resp.json()["id"]

        batch_resp = api_client.create_batch({
            "input_file_id": file_id,
            "endpoint": "/v1/chat/completions",
            "completion_window": "24h",
        }, azure_chat_batch_provider_setup["virtual_key"])
        assert batch_resp.status_code == 200
        batch = batch_resp.json()
        _assert_batch_object(batch, "/v1/chat/completions")

        get_resp = api_client.get_batch(batch["id"], azure_chat_batch_provider_setup["virtual_key"])
        assert get_resp.status_code == 200
        current = get_resp.json()
        if current["status"] in {"completed", "failed", "expired", "cancelled"}:
            pytest.skip("Batch reached terminal state before cancel could be exercised.")

        cancel_resp = api_client.cancel_batch(batch["id"], azure_chat_batch_provider_setup["virtual_key"])
        assert cancel_resp.status_code == 200, cancel_resp.text
        payload = cancel_resp.json()
        _assert_batch_object(payload, "/v1/chat/completions")
        _assert_lifecycle_surface_contract(payload)
