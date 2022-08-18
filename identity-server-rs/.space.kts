job("Identity server RS: Build and push to docker") {
    startOn {
        gitPush {
            pathFilter {
                -"README.md"
                +"*.json"
                +"dockerfile/Dockerfile"
                +"*.toml"
                +"**/*.toml"
                +"**/src/**"
            }
        }
    }

    docker {
        resources {
            cpu = 4.cpu
            memory = 3000.mb
        }
        build {
            context = "."
            file = "./docker/Dockerfile"
            labels["vendor"] = "Demius Academius from Moldova"
        }

        push("acc-md.registry.jetbrains.space/p/backend/containers/identity-server-rs") {
            tags("version-0.\$JB_SPACE_EXECUTION_NUMBER", "\$BRANCH")
        }
    }
}
