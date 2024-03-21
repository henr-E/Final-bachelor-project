# Continuous Integration & Deployment

The CI/CD in this project is achieved using the [nix package manager](https://nixos.org/).
It was chosen for a few key reasons:

- It allows the project to be reproducible anywhere. Nix allows for instance two x86_64 linux 
machines to always achieve the same result when building a package. No system dependencies are
involved.
- If it works locally, it works on the CI.
- Nix has great support for granular caching of different build steps.
- If you have already packaged something using nix it takes only a few more lines to turn this
package into a minimal docker image.

The CI can be run locally with the following command: `nix flake check -L`. The `-L` can be omitted
if you do not wish to see the logs.

It should be noted that there is one caveat to this reproducibility: it is of course impossible to
fully ensure that something is reproducible across different architectures and operating systems.

## Adding a new check to the pipeline

There are a lot of different ways to add a new check, depending on your needs. Simple checks can
be defined as follows and added to the `checks` output in `flake.nix`:
```nix
checks = {
  # Other checks omitted
  # ...

  myCheck = beLib.mkCheck {
    name = "my-check-name";
    shellScript = ''
      # Write the bash script for your check here. This example check will run neofetch and then
      # always fail. There is no need to install any of the used packages separately provided you use
      # the syntax below.
      ${pkgs.neofetch}/bin/neofetch
      exit 1
    '';
  };
};
```

## Adding a new container to release continuously

Anything added in `containers` in the `flake.nix` file will be automatically build and released
with the `nix run .#release` command used in the pipeline. Adding a container is as simple as using
`pkgs.dockerTools.buildImage` and copying a package to root:
```nix
containers = {
  # Other containers omitted
  # ...
  
  myContainer = pkgs.dockerTools.buildImage {
    name = "my-container";
    tag = "latest"; # Does not matter much in the CI, as it specified its own tags.

    copyToRoot = myPackage;
  };
};
```
There is no single way to package an application in nix. It is best to look for a language specific
tool, such as crane for rust packages or `pkgs.mkYarnPackage` for javascript packages.

To manually build a single container, simply run `nix build .#<container>-container`.

## Troubleshooting

### The check phase fails due to one or more missing files

Nix flakes by default ignore all files not added to the git source tree. `git add .` may solve
your problem.

Not solved yet?
Source files are filtered before copying them and building a package from that source. This reduces
the chance of nix needing to rebuild a package because its inputs (dependencies or source itself)
changed. To add new file paths to the filter, take a look at `sources` in `flake.nix`.
