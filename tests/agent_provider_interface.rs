use work_leaf::CodexBackend;
use work_leaf::agent::{
    AgentBackend, AgentError, AgentId, AgentKind, AgentLaunch, AgentProfile, AgentSession,
    AgentShutdownHandle, AgentStreamEvent, ChatMessage, MessageRole,
};

#[test]
fn external_agent_provider_implements_backend_through_agent_interface() {
    let profile = AgentProfile::new(
        AgentKind::External("local-provider".to_string()),
        "Local Provider",
        "local-agent",
    );
    let agent_id = AgentId::new("local-1").unwrap();
    let launch = AgentLaunch::new(
        agent_id.clone(),
        profile.kind.clone(),
        profile.default_feature.clone(),
        "answer through provider-neutral interface",
    );
    let mut backend = LocalProvider::default();
    let mut stream = Vec::new();

    let session = backend
        .launch_streaming(launch, &mut |event| stream.push(event))
        .unwrap();
    let reply = backend
        .send_streaming(&agent_id, "continue", &mut |event| stream.push(event))
        .unwrap();
    backend.shutdown_handle().shutdown();

    assert_eq!(session.id, agent_id);
    assert_eq!(session.kind, profile.kind);
    assert_eq!(session.feature, "local-agent");
    assert_eq!(session.messages[1].text, "local launch reply");
    assert_eq!(reply.text, "local send reply");
    assert_eq!(
        stream,
        vec![
            AgentStreamEvent::Status("local launch started".to_string()),
            AgentStreamEvent::AgentMessage("local launch reply".to_string()),
            AgentStreamEvent::Status("local send started".to_string()),
            AgentStreamEvent::AgentMessage("local send reply".to_string()),
        ]
    );
}

#[test]
fn codex_backend_implements_the_same_provider_neutral_trait() {
    fn assert_backend<B: AgentBackend>() {}

    assert_backend::<CodexBackend>();
}

#[derive(Default)]
struct LocalProvider {
    shutdown: AgentShutdownHandle,
}

impl AgentBackend for LocalProvider {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "local launch reply");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "local send reply"))
    }

    fn shutdown_handle(&self) -> AgentShutdownHandle {
        self.shutdown.clone()
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        sink(AgentStreamEvent::Status("local launch started".to_string()));
        sink(AgentStreamEvent::AgentMessage(
            "local launch reply".to_string(),
        ));
        self.launch(request)
    }

    fn send_streaming(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
        sink: &mut dyn FnMut(AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        sink(AgentStreamEvent::Status("local send started".to_string()));
        sink(AgentStreamEvent::AgentMessage(
            "local send reply".to_string(),
        ));
        self.send(agent_id, prompt)
    }
}
