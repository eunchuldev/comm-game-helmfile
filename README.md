# sa-kube

## Dependencies

* kubectl
* [helm](https://helm.sh/docs/intro/install/)
    * helm-diff
      `helm plugin install https://github.com/databus23/helm-diff`
    * helm-secrets
      `helm plugin install https://github.com/jkroepke/helm-secrets`
* [helmfile](https://github.com/roboll/helmfile)
* [sops](https://github.com/mozilla/sops)
* [argo-cli](https://github.com/argoproj/argo/releases)
* [kustomize](https://kubectl.docs.kubernetes.io/installation/kustomize/)
* [direnv](https://direnv.net/) - Optional


## Deploy to Dev Cluster

1. set your kubectl context to point the dev cluster
2. `NAMESPACE=<your-unique-namespace> helmfile -e dev apply`
2. Optional: create dotenv file and use direnv
  ```bash 
  cat <<EOT >> .env
  HELMFILE_ENVIRONMENT=dev
  NAMESPACE=<your-unique-namespace>
  ARGO_NAMESPACE=<your-unique-namespace>
  EOT

  # Now you can call helmfile command without parameters
  helmfile apply
  ```


## Argo workflow development

1. go to the workflow helm chart directory
  `cd charts/argo-workflow-assets/templates`
2. create workflow template as gotmpl format
3. update workflow template with `NAMESPACE=<unique-namespace> helmfile -e dev apply`
4. create and submit workflow by `argo submit -n <unique-namespace> --watch <workflow-path>`
4. Optional: create dotenv file and use direnv
  ```bash 
  cat <<EOT >> .env
  HELMFILE_ENVIRONMENT=dev
  NAMESPACE=<your-unique-namespace>
  ARGO_NAMESPACE=<your-unique-namespace>
  EOT
  ```
5. check the argo manual for advanced usage


## Update Data Access Secrets

1. create gpg key
  `gpg --generate-key`
2. send your public key to the secret owner
  `gpg --export --armor <keyname>`
3. wait until the secret owner updates the encrypted secrets and commit
  ```terminal
  # the secret owner might run below commands
  [@~]$ gpg --import pub-key.asm
  [@~]$ sed -i "$ s/$/,\n<fp>/" .sops.yaml
  [@~]$ sops updatekeys <encrypted-secret-yaml-file>
  ```
4. pull the new encrypted secrets from git and modify it
  `soap envs/dev/data-access-secrets.yaml`

Note: You can use the other method to encrypt and decript the secret(such as AWS KMS) as well. See the sops manual


## Build and Publish Images

Cluster has permission to access image registries in form of `215559030652.dkr.ecr.ap-northeast-2.amazonaws.com/sa-kube/*` 

If you place your image under `images/`, you can reference the other image on privious lexicographical order in the same directory.

you can build and publish your image to accessable registry via `./scripts/publish-images-to-ecr.sh <image-name> <tag>`.

** you can ignore the number prefix on the image name. ex) images/00_<imagename> => images/<imagename>

<TODO>
Also images under `images/` CI/CD pipeline automatically build, tag with commit hash and then deploy to the cluster
</TODO>


## Diagram

![sa-k8s](https://user-images.githubusercontent.com/6138947/103160922-1ac0e500-481e-11eb-9c2c-1d1356ccd2ff.png)
