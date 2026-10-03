package AbstructFactory_EduSphre;

// Concrete Factory 2: builds the offline family
public class OfflineCourseComponentFactory implements CourseComponentFactory {
    @Override
    public Lecture createLecture() {
        return new OfflineLecture();
    }

    @Override
    public Assessment createAssessment() {
        return new OfflineAssessment();
    }
}
