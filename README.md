## AMS - Another Messaging Service

AMS is one of a thousand messaging implementations out there. This was created for fun, but mainly as a way to teach
myself async rust. 

AMS is a peer-to-peer messaging service with optional centralized server to support authentication and message relay
between clients. When using peer-to-peer mode, clients connect directly to eachother. While users must ensure that
who they are connecting to is who they expect, messages are end-to-end encrypted.

When using client-server mode, clients connect to a central (self-hosted) AMS server. Each client must create an
account on the server, and authenticate using their credentials (simple username/password). The server supports
adding users to a contact list. Contacts will appear online / offline based on their connection status to the
server. Messages sent through the server are encrypted twice. The outer layer is encrypted for the server, which
contains the routing information and the encrypted payload. The inner layer is the encrypted payload that will be
received by the recipient and decrypted.

Some user data (usernames, contact lists, hashed passwords) are stored on the server to facilitate functionality.
However, messages are not stored on the server what-so-ever. The server simply relays the encrypted messages
between clients. If a recipient is offline, the server will attempt a retry for a configurable amount of time
before sending a delivery failure notification to the sender. At that point, the user may choose to queue the
message for later delivery upon a contact online notification event, or discard it. Choosing to queue the message
will result in the message being stored locally on the sender's device.

Please note: The server does not provide any form of user discovery. Users must know the username of the person
they wish to add. Even if a user does not exist, the server will respond as if they do, to prevent user
enumeration. The only time a user will be notified, is if the request was approved.

Clients can connect to both other clients and servers (including multiple servers) simultaneously.

## Getting started

The current state of this crate is completely unfinished and basically no functionality is working. At this point in
time, the only thing you can do is to run the binary in two instnaces, with two separate ports, and talk to eachother.
Once you've cloned the repository, it's pretty simple.

1. In terminal A: `> cargo run -b ams-unsecure <PORT1>`
2. In terminal B: `> cargo run -b ams-unsecure <PORT2>`
3. In terminal A: `connect <PORT2>`
4. In Terminal A: `send <PORT2> <MESSAGE>`
5. In terminal A: See the `<MESSAGE>` :)
