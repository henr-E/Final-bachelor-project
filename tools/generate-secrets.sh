#! /usr/bin/env bash

declare -A secrets
# Add secrets here!
# =================
secrets[DATABASE_PASSWORD]=$(openssl rand -hex 20)
secrets[JWT_SECRET]=$(openssl rand -hex 40)
# =================

# Define the folder where secrets are stored.
SECRETS_ROOT=${SECRETS_ROOT:-".secrets/"}

# Slightly hacky way to check if the current working dir is the root of the repo.
if [ ! -f "flake.nix" ]; then
  echo "Please run this command in the root of the repository."
  exit 1
fi

# Ensure the secrets folder is created.
mkdir -p $SECRETS_ROOT
# Subshell to generate secrets only if they do not yet exist.
(
  cd $SECRETS_ROOT
  # Remove obsolete secrets.
  for existing_secret in $(find . -mindepth 1)
  do
    trimmed=${existing_secret:2}
    if [[ ! -v "secrets[$trimmed]" ]]; then
      echo "Removing unknown secret: \`$trimmed\`"
      rm $existing_secret
    fi
  done
  # Generate secrets that do not yet exists.
  for secret_name in "${!secrets[@]}"
  do
    if [ ! -f $secret_name ]; then
      echo "Generating new secret: \`$secret_name\`"
      echo -n ${secrets[$secret_name]} >> $secret_name
    else
      echo "Secret exists: \`$secret_name\`"
    fi
  done
)
