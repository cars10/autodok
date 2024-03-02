# autodok

> restart your containers with a new image

*autodok* provides an API to restart your running docker containers with a new image.  
It is similar to [watchtower](https://containrrr.dev/watchtower/), but it does not poll for new images - *autodok* only updates images & containers on-demand.

It is intended to be triggered by your CI pipeline, the recommended workflow is:

1. You push new code
2. This triggers your CI to build & push a new image
3. Your CI sends a `POST` request to autodok with the name of your running container and the new image:tag 
4. *autodok* will download the new image and restart the container with the new image


## How to

### Setup

You can run *autodok* as a docker container with docker compose:

1. copy the included `compose.yml` to your server
2. make the service available by either uncommenting the port mapping or by exposing *autodok* via a reverse proxy of your choice (nginx, traefik, etc., **recommended**)
3. run `docker compose up -d`

You should check the logs (`docker compose logs -f autodok`) and copy the provided API key.

### Usage

#### Manually

Trigger *autodok* manually by sending a `POST` request to `/update`, e.g. to restart the container `foo` with the image `foo:latest`:

```bash
curl -X POST "http://autodok.example.com/update" \
     -H "Authorization: $API_KEY" \
     -H "content-type: application/json" \
     -d '{"container": "foo", "image": "foo:latest"}'
```

#### CI

You can send requests to *autodok* however you like. Run a bash script, write a custom application, etc. An example with github actions could look like this:

```yaml
  - name: trigger autodok deployment
    uses: fjogeleit/http-request-action@v1
    with:
      url: 'http://autodok.example.com/update'
      timeout: 300000
      method: 'POST'
      customHeaders: '{"Content-Type": "application/json", "Authorization": "${{ secrets.AUTODOK_API_KEY }}"}'
      data: '{"container": "foo", "image": "foo:latest"}'
```

For this to work you need to add a new secret (settings / secrets / actions / repository secrets) named `AUTODOK_API_KEY` that is set to the correct value.

## API

Some endpoints in *autodok* require authorization. You have to send an `Authorization` header with the provided API key. The key will be logged each time when you start *autodok*, and you can always read it from `./data/api_key`.

### Endpoints

*autodok* exposes two endpoints:

#### `GET /health`

This is a simple healthcheck that returns `200 OK` if the docker daemon is reachable. It does not require authorization and is always available.

#### `POST /update`

This endpoint will check if a container with the provided name is running. If it is, it will download the provided image and restart the container with the new image and the same settings (volumes etc.) as before.  
You have to send an `Authorization` header with the correct API key to use this endpoint.

The following fields are supported in the request body:

| field                  | example    | required                                        | description                         |
|------------------------|------------|-------------------------------------------------|-------------------------------------|
| container              | foo        | yes                                             | Name of a **running** container     |
| image                  | foo:latest | yes                                             | Name and tag of an existing image   |

### Examples

Healthcheck 
```bash
curl -X GET "http://autodok.example.com/health"
```

Update container `elasticsearch` with new image `docker.elastic.co/elasticsearch/elasticsearch:8.8.0`
```bash
curl -X POST "http://autodok.example.com/update" \
     -H "Authorization: $API_KEY" \
     -H "content-type: application/json" \
     -d '{"container": "elasticsearch", "image": "docker.elastic.co/elasticsearch/elasticsearch:8.8.0"}'

# response:
{"message":"Container 'elasticsearch' restarted with new image 'docker.elastic.co/elasticsearch/elasticsearch:8.8.0'"}
```

## FAQ & possible issues

### Request timeouts

Requests to `POST /update` might timeout depending on your setup. Please keep in mind that downloading and restarting a big docker image might take a few minutes - your timeout settings have to respect this. Remember to set timeouts for your proxy server **and** for the client that executes the request, e.g. the used github action.

### Using a private registry

You can download images from private registries by providing a standard docker `config.json`. Uncomment the volume in `compose.yml` and adjust the path to your `config.json`. *autodok* will then use your existing credentials whenever needed.

### Restarting multiple containers at once

If you want to restart multiple containers you have to execute multiple requests, each containing the respective container name.

## License

MIT