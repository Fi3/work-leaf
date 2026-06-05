## Terminal based ui

* it use vim control keys for everything
* on the left (1/5 of tot space) you see all the agents that you are running, and you can select
    work-leaf command interface
* on the right you have the chat with the selected agent, or the work-leaf command interface
* each agent chat (left part) have introspection, you can see what is working on, all the
    files that modified, which are the other agents that are modifing the same files, if the
    agents patch depends on others agents, or if other agents depend on it, every agent is
    clickable so it open the specific agent chat
* right part can be hide/show cliking "," when in command mode
* to go in command press (esc), to go in insert mode press (i)
* when the cursor is in the right part we are always in command mode
* ctrl-w (when in command mode) then (h) go left or (l) go right
* (s) open the chate in the same pane (it spplit the left part in 2 3 ecc ecc so can open
    many chat)
* (t) open the chat in a new window (gt go next window gT prev window)
* when on command mode and cursor is over a chat-id (f) will fork that chat
* on the left where you see all the agents if an agent is ready you will see it highlighted


## File locking

* the orchestrator inject in EVERY prompt a command where it say that:
    * the agent is not allowed to read from files but it must ask orchestrator to provide
        the file text
    * the agent is not allowed to write
    to files but, the agent must provide a patch for every file that want to write
    * the second point means that also all commands that result in writing, that means the
        we need a list of all the majior programming languages their command to build and
        which file are going to written so the oechestrator know if a build command can be
        release or not.
    * for each file the orchestrator keep an internal lock, the lock is an RwLock, it can unlock
        the file to many reader or to one writer at time

## Patching

* when the orchestrator receive a patch request it lock the files in write mode (wait until
    is possible) and try to apply the patch (use git to do these things) if there are no
    conflict apply the patch if there are conflict ask the agent to fix the patch
* when a patch is applied the code is commited (don't care if do not compile or wahtever),
    the agent id is commited, the broad feature that the agent is working on is commited,
    and the specific reason of that patch is commited

## Reviews

* when feature is ready, or whenever we decided that code is fine. We ask the orchestrator to
    review. The orcherstrator will go trough all the commit history, and it will find the
    final commit for every chat-id, will spawn a new agent for every unique chat id, ask the
    agent for that chat id to summarize what the patch does and ask the newly created agent to
    review it.
* every review is fed to the original agent that did the patch so that can be fixed.
* when fixed orchestrator will ask the reviewer to check the path again
* and so on until no findings left for all the chats

## Linearize

* after the review process the orchestrator will ask the user for every chat-id if the patch
    from that chat should result in a commit or it should be integrated into other commits
* it also ask if commits from specific chat-ids should be grouped in only one commit
* this information are passed to a new agent that will merge all commits from the
    chat-ids that we want to keep into only one, and it will merge all commits of chat-ids that
    we do not want to keep into the ones that make most sense. It will then iterate trhoug test
    (or whatver is defined in the repo instruction) and try to make the diff with master/main as
    small as possible.

## command chat

## nvim integration

## database
