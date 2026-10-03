package AbstructFactory_EduSphre;

// Client: knows only the abstract factory and the abstract products
public class EduSphereCourse {
    private final Lecture lecture;
    private final Assessment assessment;

    public EduSphereCourse(CourseComponentFactory factory) {
        this.lecture = factory.createLecture();
        this.assessment = factory.createAssessment();
    }

    public void runCourseWorkflow() {
        lecture.deliver();
        lecture.shareMaterials();
        assessment.conduct();
        assessment.publishResults();
    }
}
