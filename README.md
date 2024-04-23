# Fast Multi-User In-Memory Logging Management Service

> Important: Work in progress, don't use in a production environment unless you know what you're doing.

Simple and fast in-memory structure-agnostic logging management service designed for services that need
to manage logs coming from multiple users.

> The service must be run locally, any unauthorized access to the endpoints will lead to malicious users logging on behalf of other users. 
> Make sure you are protected against SSRF if running along with other services.

## Functionality

The service boils down to operating on a shared key-value store where the key is the user id and the value is the
user logs object.

The service supports adding logs that implement a `IsLog` trait through a unified endpoint `log/{user_id}`. The
service then understands which kind of log was submitted based on the log level, then loads it in the shared
state assigning it also the associated system time. 

Logs can either be retrieved in their own generic format wrapped in the `LogWrapper` object by specifying
the error type, or through a unified endpoint that returns a `Vec<ServiceLog>` view.

Note also that logging for a user can be turned on and off using `is_logging()` or `is_not_logging()`.
