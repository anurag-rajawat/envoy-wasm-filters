# Envoy Wasm HTTP Filter

This is an HTTP filter that will observe the API calls made to/from a k8s workload.

## Samples:

- Simple API Event:
  ```json
  {
    "context_id": 2,
    "request": {
      "headers": {
        "user-agent": "Wget",
        ":path": "/",
        ":scheme": "http",
        "x-request-id": "183d4792-7d2c-9559-b9c7-60270f37a25a",
        "x-envoy-decorator-operation": "user-svc.prod.svc.cluster.local:8080/*",
        ":authority": "user-svc.prod:8080",
        "x-envoy-peer-metadata": "ChkKDkFQUF9DT05UQUlORVJTEgcaBWh0dHBkChoKCkNMVVNURVJfSUQSDBoKS3ViZXJuZXRlcwodCgxJTlNUQU5DRV9JUFMSDRoLMTAuMjQ0LjAuMTgKGQoNSVNUSU9fVkVSU0lPThIIGgYxLjIxLjIKoQEKBkxBQkVMUxKWASqTAQoOCgNhcHASBxoFaHR0cGQKJAoZc2VjdXJpdHkuaXN0aW8uaW8vdGxzTW9kZRIHGgVpc3RpbwoqCh9zZXJ2aWNlLmlzdGlvLmlvL2Nhbm9uaWNhbC1uYW1lEgcaBWh0dHBkCi8KI3NlcnZpY2UuaXN0aW8uaW8vY2Fub25pY2FsLXJldmlzaW9uEggaBmxhdGVzdAoaCgdNRVNIX0lEEg8aDWNsdXN0ZXIubG9jYWwKHwoETkFNRRIXGhVodHRwZC1jNmQ2Y2I5NGItN3N3OGoKFgoJTkFNRVNQQUNFEgkaB2RlZmF1bHQKSQoFT1dORVISQBo+a3ViZXJuZXRlczovL2FwaXMvYXBwcy92MS9uYW1lc3BhY2VzL2RlZmF1bHQvZGVwbG95bWVudHMvaHR0cGQKGAoNV09SS0xPQURfTkFNRRIHGgVodHRwZA==",
        ":method": "GET",
        "x-envoy-peer-metadata-id": "sidecar~10.244.0.18~httpd-c6d6cb94b-7sw8j.default~default.svc.cluster.local",
        "x-forwarded-proto": "http"
      },
      "body": ""
    },
    "response": {
      "headers": {
        "x-envoy-peer-metadata-id": "sidecar~10.244.0.17~rest-api-66df6cbbfc-c6rd2.prod~prod.svc.cluster.local",
        "date": "Wed, 04 Sep 2024 06:14:54 GMT",
        "server": "istio-envoy",
        ":status": "308",
        "content-length": "33",
        "x-envoy-upstream-service-time": "14",
        "content-type": "application/json; charset=utf-8",
        "x-envoy-peer-metadata": "ChwKDkFQUF9DT05UQUlORVJTEgoaCHJlc3QtYXBpChoKCkNMVVNURVJfSUQSDBoKS3ViZXJuZXRlcwodCgxJTlNUQU5DRV9JUFMSDRoLMTAuMjQ0LjAuMTcKGQoNSVNUSU9fVkVSU0lPThIIGgYxLjIxLjIKpwEKBkxBQkVMUxKcASqZAQoRCgNhcHASChoIcmVzdC1hcGkKJAoZc2VjdXJpdHkuaXN0aW8uaW8vdGxzTW9kZRIHGgVpc3RpbwotCh9zZXJ2aWNlLmlzdGlvLmlvL2Nhbm9uaWNhbC1uYW1lEgoaCHJlc3QtYXBpCi8KI3NlcnZpY2UuaXN0aW8uaW8vY2Fub25pY2FsLXJldmlzaW9uEggaBmxhdGVzdAoaCgdNRVNIX0lEEg8aDWNsdXN0ZXIubG9jYWwKIwoETkFNRRIbGhlyZXN0LWFwaS02NmRmNmNiYmZjLWM2cmQyChMKCU5BTUVTUEFDRRIGGgRwcm9kCkkKBU9XTkVSEkAaPmt1YmVybmV0ZXM6Ly9hcGlzL2FwcHMvdjEvbmFtZXNwYWNlcy9wcm9kL2RlcGxveW1lbnRzL3Jlc3QtYXBpChsKDVdPUktMT0FEX05BTUUSChoIcmVzdC1hcGk="
      },
      "body": "{\"message\":\"Namaste World! 🙏\"}"
    },
    "source": {
      "name": "httpd-c6d6cb94b-7sw8j",
      "namespace": "default",
      "ip": "10.244.0.18",
      "port": 37704
    },
    "destination": {
      "name": "rest-api-66df6cbbfc-c6rd2",
      "namespace": "prod",
      "ip": "10.96.18.127",
      "port": 8080
    }
  }
  ```

- Simple API Event with request and response bodies:
  ```json
  {
    "context_id": 3,
    "request": {
      "headers": {
        ":path": "/v1/signin",
        "x-forwarded-proto": "http",
        "x-envoy-peer-metadata-id": "sidecar~10.244.0.18~httpd-c6d6cb94b-7sw8j.default~default.svc.cluster.local",
        "user-agent": "curl/8.5.0",
        "x-envoy-peer-metadata": "ChkKDkFQUF9DT05UQUlORVJTEgcaBWh0dHBkChoKCkNMVVNURVJfSUQSDBoKS3ViZXJuZXRlcwodCgxJTlNUQU5DRV9JUFMSDRoLMTAuMjQ0LjAuMTgKGQoNSVNUSU9fVkVSU0lPThIIGgYxLjIxLjIKoQEKBkxBQkVMUxKWASqTAQoOCgNhcHASBxoFaHR0cGQKJAoZc2VjdXJpdHkuaXN0aW8uaW8vdGxzTW9kZRIHGgVpc3RpbwoqCh9zZXJ2aWNlLmlzdGlvLmlvL2Nhbm9uaWNhbC1uYW1lEgcaBWh0dHBkCi8KI3NlcnZpY2UuaXN0aW8uaW8vY2Fub25pY2FsLXJldmlzaW9uEggaBmxhdGVzdAoaCgdNRVNIX0lEEg8aDWNsdXN0ZXIubG9jYWwKHwoETkFNRRIXGhVodHRwZC1jNmQ2Y2I5NGItN3N3OGoKFgoJTkFNRVNQQUNFEgkaB2RlZmF1bHQKSQoFT1dORVISQBo+a3ViZXJuZXRlczovL2FwaXMvYXBwcy92MS9uYW1lc3BhY2VzL2RlZmF1bHQvZGVwbG95bWVudHMvaHR0cGQKGAoNV09SS0xPQURfTkFNRRIHGgVodHRwZA==",
        "x-request-id": "56e1acc9-b97f-9552-b584-6bb856b27e67",
        ":method": "POST",
        ":authority": "user-svc.prod:8080",
        ":scheme": "http",
        "x-envoy-decorator-operation": "user-svc.prod.svc.cluster.local:8080/*",
        "accept": "*/*",
        "content-type": "application/json",
        "content-length": "48"
      },
      "body": "{\"email\": \"abc@email.com\", \"password\": \"abcdef\"}"
    },
    "response": {
      "headers": {
        "content-length": "191",
        "x-envoy-peer-metadata": "ChwKDkFQUF9DT05UQUlORVJTEgoaCHJlc3QtYXBpChoKCkNMVVNURVJfSUQSDBoKS3ViZXJuZXRlcwodCgxJTlNUQU5DRV9JUFMSDRoLMTAuMjQ0LjAuMTcKGQoNSVNUSU9fVkVSU0lPThIIGgYxLjIxLjIKpwEKBkxBQkVMUxKcASqZAQoRCgNhcHASChoIcmVzdC1hcGkKJAoZc2VjdXJpdHkuaXN0aW8uaW8vdGxzTW9kZRIHGgVpc3RpbwotCh9zZXJ2aWNlLmlzdGlvLmlvL2Nhbm9uaWNhbC1uYW1lEgoaCHJlc3QtYXBpCi8KI3NlcnZpY2UuaXN0aW8uaW8vY2Fub25pY2FsLXJldmlzaW9uEggaBmxhdGVzdAoaCgdNRVNIX0lEEg8aDWNsdXN0ZXIubG9jYWwKIwoETkFNRRIbGhlyZXN0LWFwaS02NmRmNmNiYmZjLWM2cmQyChMKCU5BTUVTUEFDRRIGGgRwcm9kCkkKBU9XTkVSEkAaPmt1YmVybmV0ZXM6Ly9hcGlzL2FwcHMvdjEvbmFtZXNwYWNlcy9wcm9kL2RlcGxveW1lbnRzL3Jlc3QtYXBpChsKDVdPUktMT0FEX05BTUUSChoIcmVzdC1hcGk=",
        ":status": "200",
        "content-type": "application/json; charset=utf-8",
        "server": "istio-envoy",
        "date": "Wed, 04 Sep 2024 06:16:53 GMT",
        "x-envoy-upstream-service-time": "95",
        "set-cookie": "Authorization=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6ImFiY0BlbWFpbC5jb20iLCJleHBpcmVzX2F0IjoxNzI1NTE3MDEzLCJpZCI6MX0.uYn1des05sPjzQ9Q_mV27m-xJLh_NO5mSjbwi0sC17I; Path=/v1/signin; Max-Age=86400; HttpOnly",
        "x-envoy-peer-metadata-id": "sidecar~10.244.0.17~rest-api-66df6cbbfc-c6rd2.prod~prod.svc.cluster.local"
      },
      "body": "{\"access_token\":\"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6ImFiY0BlbWFpbC5jb20iLCJleHBpcmVzX2F0IjoxNzI1NTE3MDEzLCJpZCI6MX0.uYn1des05sPjzQ9Q_mV27m-xJLh_NO5mSjbwi0sC17I\",\"type\":\"bearer\"}"
    },
    "source": {
      "name": "httpd-c6d6cb94b-7sw8j",
      "namespace": "default",
      "ip": "10.244.0.18",
      "port": 55440
    },
    "destination": {
      "name": "rest-api-66df6cbbfc-c6rd2",
      "namespace": "prod",
      "ip": "10.96.18.127",
      "port": 8080
    }
  }
  ```

# Getting Started

## Install development tools

You'll need these tools for a smooth development experience:

- [Make](https://www.gnu.org/software/make/#download),
- [Rust](https://www.rust-lang.org/tools/install) toolchain,
- An IDE ([RustRover](https://www.jetbrains.com/rust/) / [VS Code](https://code.visualstudio.com/download)),
- Container tools ([Docker](https://www.docker.com/) / [Podman](https://podman.io/)),
- [Kubernetes cluster](https://kubernetes.io/docs/setup/) running version 1.26 or later,
- [kubectl](https://kubernetes.io/docs/tasks/tools/#kubectl) version 1.26 or later.

## In Envoy alone

This example can be run with docker compose and has a matching [envoy configuration](envoy.yaml) file.

- Install the rust wasm toolchain:
  ```shell
  rustup target add wasm32-unknown-unknown
  ```

- Build the plugin
  ```shell
  make
  ```

- Start the envoy container
  ```shell
  docker compose up
  ```

- See the Raw API Events in `server` cluster configured in [envoy configuration](envoy.yaml).

# In Kubernetes

- [Install Istio](https://istio.io/latest/docs/setup/install/)
- Update the `.spec.configPatches[0].patch.value.typedConfig.value.config.configuration.value` value according
  to `api_path`
  exposed by configured upstream cluster.
- Create the [envoy filter](auto-observe-filter.yaml) to observe the API calls:

  ```shell
  kubectl apply -f auto-observe-filter.yaml
  ```

- Enable the envoy proxy injection by labeling the namespace in which you'll deploy workload:
  ```shell
  kubectl label ns <namespace_name> istio-injection=enabled
  ```
- Deploy some workload and generate traffic by calling some APIs.
- See the observed Raw API Events in `filterserver` cluster configured in [envoy filter](auto-observe-filter.yaml).
