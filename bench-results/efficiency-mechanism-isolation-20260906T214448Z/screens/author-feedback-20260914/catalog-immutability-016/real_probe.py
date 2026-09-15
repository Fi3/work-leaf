"""One separately admitted immutable-view integration-input diagnostic."""
import provider_route
import integration_probe

integration_probe.command = provider_route.probe_command
integration_probe.main()
