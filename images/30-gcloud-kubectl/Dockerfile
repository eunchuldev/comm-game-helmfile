FROM gcr.io/google.com/cloudsdktool/cloud-sdk:alpine

# this command increases image size +800mb(what happened?)
# RUN gcloud components install kubectl

ARG KUBE_VERSION="v1.20.2"
ADD https://storage.googleapis.com/kubernetes-release/release/${KUBE_VERSION}/bin/linux/amd64/kubectl /usr/bin/kubectl
RUN chmod +x /usr/bin/kubectl
