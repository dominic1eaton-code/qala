qala: solution factory operating system

qala is a solution factory operating system where users create solutions and manage their full lifecycle.

solution types
- application
- system
- good
- product
- service
- platform

solution structure model
- system contains applications
- application contains processes
- process contains components
- component contains interfaces
- interface has imports and exports
- interface contains messages
- message type is either:
  - event (dynamic)
  - state (static)
- messages can be imported and exported through interfaces
- message contains data structures
- data structure contains data

data types
- bool
- string
- int
- custom
- varchar
- float
- double
- null
- array
- char
- tuple
- set
- map
- object
- pointer
- date

sde: solution development environment
- an SDE is deployable, configurable, and distributable
- an SDE can be deployed to a platform, environment, or machine
- an SDE is solution-agnostic and defines the conditions for:
  - creation
  - maintenance
  - lifecycle management
  - operational management
  - solution replication
- an SDE includes:
  - envirnoment variables
  - IDEs
  - languages
  - user preferences
  - configuration
  - profiles
  - settings
  - parameters
  - options
  - toolkits which contain toolsets which contain tools, which when tools are linked together, they form toolchains
  - solution documentation, charters, documents, files
  - solution content management system
- one SDE can support a collection of solutions

sf: solution factory
- an SF is a coordinated collection of connected SDEs
- SDEs in an SF are networked and orchestrated
- the SF produces solutions in a consistent, repeatable, and scalable way
