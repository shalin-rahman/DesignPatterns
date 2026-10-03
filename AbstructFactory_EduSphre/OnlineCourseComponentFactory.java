package AbstructFactory_EduSphre;

// Concrete Factory 1: builds the online family
public class OnlineCourseComponentFactory implements CourseComponentFactory {
    @Override
    public Lecture createLecture() {
        return new OnlineLecture();
    }

    @Override
    public Assessment createAssessment() {
        return new OnlineAssessment();
    }
}
