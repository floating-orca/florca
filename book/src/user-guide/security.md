# Security

## Authentication and authorization

<div class="warning">

**Currently, one instance of _FloatingOrca_ is not meant to be shared by multiple users.** This is because neither the engine nor the deployer have any proper authentication or authorization mechanisms in place.

</div>

The only security measure in place is Basic Authentication configured on the reverse proxy that sits in front of these services.
The sole purpose of this is to prevent unauthorized access to the system when running on a public server.
The reason for basic authentication is that it is simple to set up and browsers can handle it without any additional configuration.

## Plugin execution

A single workflow with all its plugin functions is executed in a single Deno process, run with `--allow-all` permissions.

<div class="warning">

**Make sure to only run trusted code in plugin functions**, as they have access to various system resources, including the file system and network.

</div>

_Depending on whether you run the engine in a container or not, access to the file system and network may be limited to the container only._
