package AbstructFactory_EduSphre;

// Concrete Product B2
public class OfflineAssessment implements Assessment {
    @Override
    public void conduct() {
        System.out.println("Conducting written exam on paper");
    }

    @Override
    public void publishResults() {
        System.out.println("Posting manually graded results on the notice board");
    }
}
