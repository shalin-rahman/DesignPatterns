package AbstructFactory_EduSphre;

// Concrete Product B1
public class OnlineAssessment implements Assessment {
    @Override
    public void conduct() {
        System.out.println("Conducting MCQ quiz on the online portal");
    }

    @Override
    public void publishResults() {
        System.out.println("Publishing auto-graded results on student dashboards");
    }
}
