# API Quick Reference

## Commands

### Person Management
```rust
// Create a new person
CreatePerson { 
    id: PersonId, 
    name: String 
}

// Update person name
UpdatePersonName { 
    id: PersonId, 
    name: String 
}

// Lifecycle management
ActivatePerson { id: PersonId }
SuspendPerson { id: PersonId }
ArchivePerson { id: PersonId }
```

### Component Management
```rust
// Contact Information
AddContactInfo { 
    person_id: PersonId, 
    contact: ContactInfo 
}
UpdateContactInfo { 
    person_id: PersonId, 
    contact: ContactInfo 
}

// Skills
AddSkill { 
    person_id: PersonId, 
    skill: Skill 
}
UpdateSkillLevel { 
    person_id: PersonId, 
    skill_id: SkillId, 
    level: SkillLevel 
}

// Preferences
SetPreferences { 
    person_id: PersonId, 
    preferences: Preferences 
}
```

## Queries

### Person Queries
```rust
// Get person by ID
get_person(person_id: PersonId) -> Option<Person>

// Search persons
search_persons(criteria: SearchCriteria) -> Vec<PersonSummary>

// Get person with components
get_person_with_components(person_id: PersonId) -> PersonView
```

### Component Queries
```rust
// Get specific components
get_contact_info(person_id: PersonId) -> Option<ContactInfo>
get_skills(person_id: PersonId) -> Vec<Skill>
get_preferences(person_id: PersonId) -> Option<Preferences>
```

### Projection Queries
```rust
// Network view
get_person_network(person_id: PersonId) -> NetworkView

// Skills summary
get_skills_summary(person_id: PersonId) -> SkillsSummary

// Timeline
get_person_timeline(person_id: PersonId) -> Timeline
```

## Events

### Core Events
```rust
PersonCreated { id: PersonId, name: String }
PersonNameUpdated { id: PersonId, name: String }
PersonActivated { id: PersonId }
PersonSuspended { id: PersonId }
PersonArchived { id: PersonId }
```

### Component Events
```rust
ContactInfoAdded { person_id: PersonId, contact: ContactInfo }
SkillAdded { person_id: PersonId, skill: Skill }
PreferencesSet { person_id: PersonId, preferences: Preferences }
```

### Cross-Domain Events
```rust
// Location integration
PersonLocationAssigned { person_id: PersonId, location_id: LocationId }

// Organization integration
PersonEmploymentAdded { person_id: PersonId, org_id: OrganizationId, role: String }

// Identity integration
PersonIdentityLinked { person_id: PersonId, identity_id: IdentityId }
```

## Common Patterns

### Creating a Person with Components
```rust
// 1. Create person
let cmd = CreatePerson { id, name };
command_bus.send(cmd).await?;

// 2. Add components
let contact_cmd = AddContactInfo { person_id: id, contact };
command_bus.send(contact_cmd).await?;

let skill_cmd = AddSkill { person_id: id, skill };
command_bus.send(skill_cmd).await?;
```

### Querying Person Data
```rust
// Get complete person view
let person_view = query_bus
    .query(GetPersonWithComponents { person_id })
    .await?;

// Get specific projection
let network = query_bus
    .query(GetPersonNetwork { person_id })
    .await?;
```