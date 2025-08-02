#\!/bin/bash

# Add event_type method to each event implementation
sed -i '/impl NixDomainEvent for FlakeUpdated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "FlakeUpdated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for FlakeInputAdded {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "FlakeInputAdded"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for PackageBuilt {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "PackageBuilt"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for ModuleCreated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "ModuleCreated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for OverlayCreated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "OverlayCreated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for ConfigurationCreated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "ConfigurationCreated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for ConfigurationActivated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "ConfigurationActivated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for ExpressionEvaluated {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "ExpressionEvaluated"/' src/events/mod.rs

sed -i '/impl NixDomainEvent for GarbageCollected {/,/^}$/ s/    fn as_any(&self) -> &dyn Any {/    fn as_any(\&self) -> \&dyn Any {\n        self\n    }\n    \n    fn event_type(\&self) -> \&'"'"'static str {\n        "GarbageCollected"/' src/events/mod.rs

