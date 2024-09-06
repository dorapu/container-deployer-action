# Container Deployer Action

A custom GitHub Action container deployer for Dorapu.

## How to Use

Add `docker://ghcr.io/dorapu/container-deployer-action:v0.1.0` to the `uses` field of the job step.
Then, specify the `CONFIG_FILE` environment variable to tell the deployer the name of the config file.
We may use `deploy.toml` as the config file. It must be a TOML file by the way.

### Deployer Environment Variables

| Variable Name | Data Type | Required | Default Value |
|---------------|-----------|----------|---------------|
| CONFIG_FILE   | String    | Yes      |               |
| DRY_RUN       | Boolean   | No       | false         |

**Notes**:
- Dry run mode is useful for testing the `docker build` command. May be used for the CI workflow.
- Add any other environment variables as needed. For example, we need to add three more for GHCR deployment: Hostname, username, and password.

### Workflow Example

```yaml
name: CI
run-name: running dry run mode for ${{ github.sha }} by @${{ github.actor }}
on:
  pull_request:
    branches:
      - master
jobs:
  dry-run:
    runs-on: ubuntu-latest
    steps:
      - name: check out repository code
        uses: actions/checkout@v4
      - name: running dry run mode
        uses: docker://ghcr.io/dorapu/container-deployer-action:v0.1.0
        env:
          CONFIG_FILE: deploy.toml
          DRY_RUN: true
          GHCR_HOSTNAME: ${{ vars.GHCR_HOSTNAME }}
          GHCR_USERNAME: ${{ vars.GHCR_USERNAME }}
          GHCR_PASSWORD: ${{ secrets.GHCR_PASSWORD }}
```

### Run Example

```sh
checking whether docker command exists... it is.
config file is set to "deploy.toml".
collecting all deploy.toml file in this repository... found 3:
  -> /github/workspace/deploy.toml
  -> /github/workspace/examples/example123/deploy.toml
  -> /github/workspace/examples/example456/deploy.toml
reading config from "/github/workspace/deploy.toml"... done.
reading config from "/github/workspace/examples/example123/deploy.toml"... done.
reading config from "/github/workspace/examples/example456/deploy.toml"... done.
starting the deployment process for "ghcr" registry:
  -> logging in to "ghcr.io" with username "wisn":
    -> echo <REDACTED> | docker login ghcr.io -u wisn --password-stdin
    -> done (Login Succeeded)
  -> processing image "container-deployer-action" with tag "latest":
    -> docker build -q -t ghcr.io/dorapu/container-deployer-action -f Dockerfile .
    -> done (sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de)
    -> docker tag sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de ghcr.io/dorapu/container-deployer-action:latest
    -> done
    -> docker push ghcr.io/dorapu/container-deployer-action:latest
    -> done
    -> docker rmi sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de -f
    -> done
  -> processing image "container-deployer-action" with tag "v0.1.0":
    -> docker build -q -t ghcr.io/dorapu/container-deployer-action -f Dockerfile .
    -> done (sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de)
    -> docker tag sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de ghcr.io/dorapu/container-deployer-action:v0.1.0
    -> done
    -> docker push ghcr.io/dorapu/container-deployer-action:v0.1.0
    -> done
    -> docker rmi sha256:a64de0c0dc2159f216587bef19d3773eaece5ea4b88dd266e7f823834bc740de -f
    -> done
  -> processing image "example123" with tag "ignored":
    -> set as to ignore by the config. ignored.
  -> processing image "example123" with tag "latest":
    -> docker build -q -t ghcr.io/dorapu/example123 -f Containerfile .
    -> done (sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14)
    -> docker tag sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14 ghcr.io/dorapu/example123:latest
    -> done
    -> docker push ghcr.io/dorapu/example123:latest
    -> done
    -> docker rmi sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14 -f
    -> done
  -> processing image "example123" with tag "v2024.9.5":
    -> will not publish since tag already exist. "replace" field config is set to false.
  -> processing image "example456" with tag "latest":
    -> docker build -q -t ghcr.io/dorapu/example456 -f Dockerfile .
    -> done (sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14)
    -> docker tag sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14 ghcr.io/dorapu/example456:latest
    -> done
    -> docker push ghcr.io/dorapu/example456:latest
    -> done
    -> docker rmi sha256:aa69c84389db9f715076d36e2ffb1cecd9adb5ce2b08fbc98ff7256ce2b22d14 -f
    -> done
all done.
```

## Deployment Config File

The name of the config file can be anything. However, the file format must be in TOML.

### Configuration Anatomy

```toml
[registries.<registry-name>] # define the registry name, for example, ghcr
hostname = "GHCR_HOSTNAME" # put the environment variable for the hostname
username = "GHCR_USERNAME" # put the environment variable for the username
password = "GHCR_PASSWORD" # put the environment variable for the password

[registries.<registry-name>.images.<image-identifier>] # define the image definition. image identifier can be anything. for example, latest
source = "Dockerfile" # the source (file name) to assemble the image
repository = "dorapu" # the target repository name
name = "container-deployer-action" # the name of the image to be uploaded
tag = "latest" # the tag of the image to be uploaded
replace = true # force upload image even if the tag is already exist. if set to false, will not replace existing tag. default is false
ignore = true # ignore this image definition. will not build and no publish the image. default is false
```

### Configuration Example

```toml
[registries.ghcr]
hostname = "GHCR_HOSTNAME"
username = "GHCR_USERNAME"
password = "GHCR_PASSWORD"

[registries.ghcr.images.latest]
source = "Containerfile"
repository = "dorapu"
name = "example123"
tag = "latest"
replace = true

[registries.ghcr.images.tagged]
source = "Containerfile"
repository = "dorapu"
name = "example123"
tag = "v2024.9.5"

[registries.ghcr.images.ignored]
ignore = true
repository = "dorapu"
source = "Containerfile"
name = "example123"
tag = "ignored"
```

## License

This project is licensed under the [MIT license](LICENSE).
