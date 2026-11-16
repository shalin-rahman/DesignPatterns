# Implementation Plan: Refactoring Guru EventManager Observer Design Pattern

## Objective
Implement the Observer design pattern in Java based on Refactoring Guru's EventManager / Editor example. The implementation includes clear architectural role separation, understanding comments, comprehensive documentation, UML diagrams, and execution verification.

## Architecture & Components
1. **Observer Interface (`EventListener`)**:
   - Declares common update method: `void update(String eventType, File file)`.
2. **Subject / Event Publisher (`EventManager`)**:
   - Manages subscriber subscriptions using a hash map keyed by event type (`Map<String, List<EventListener>>`).
   - Supports `subscribe(String eventType, EventListener listener)`, `unsubscribe(String eventType, EventListener listener)`, and `notify(String eventType, File file)`.
3. **Context / Concrete Publisher (`Editor`)**:
   - Encapsulates file operations (`openFile(String filePath)`, `saveFile()`).
   - Owns a public instance of `EventManager` initialized with specific event types (`open`, `save`).
4. **Concrete Observer A (`EmailNotificationListener`)**:
   - Implements `EventListener`.
   - Simulates sending notification emails to a configured address when triggered.
5. **Concrete Observer B (`LogOpenListener`)**:
   - Implements `EventListener`.
   - Simulates writing audit log entries to a designated file when triggered.
6. **Driver (`Main`)**:
   - Orchestrates the components, registers listeners, triggers file operations, demonstrates unsubscribing, and verifies notification dispatches.

## Execution Steps
1. **Create `PLAN.md`**: Define the architecture, atomic milestones, and deliverables.
2. **Implement Source Files**:
   - `EventListener.java`: Observer interface contract.
   - `EventManager.java`: Event publisher and subscriber registry.
   - `Editor.java`: Business context performing file operations and dispatching notifications.
   - `EmailNotificationListener.java`: Email notification subscriber.
   - `LogOpenListener.java`: Audit log subscriber.
   - `Main.java`: Execution driver demonstrating subscription lifecycle and events.
3. **Compilation & Testing**:
   - Compile all `.java` files using `javac`.
   - Execute `java Main` and assert expected behavior.
4. **Documentation & UML Diagrams**:
   - Generate `uml-class-diagram.svg` and `uml-sequence-diagram.svg`.
   - Generate `README.md` with component mapping, design principles, Mermaid diagrams, and output traces.
