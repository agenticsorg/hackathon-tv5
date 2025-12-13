import XCTest
@testable import VibeCheck

final class HealthKitManagerTests: XCTestCase {

    var manager: HealthKitManager!

    override func setUp() {
        super.setUp()
        manager = HealthKitManager()
    }

    override func tearDown() {
        manager = nil
        super.tearDown()
    }

    func testInitializationDefaults() {
        XCTAssertNil(manager.currentHRV)
        XCTAssertNil(manager.lastSleepHours)
        XCTAssertNil(manager.restingHeartRate)
        XCTAssertNil(manager.stepsToday)
        XCTAssertEqual(manager.activityLevel, .unknown)
        XCTAssertFalse(manager.isLoading)
        XCTAssertFalse(manager.isAuthorized)
        XCTAssertNil(manager.errorMessage)
    }

    func testActivityClassification() {
        // Sedentary < 2000
        XCTAssertEqual(manager.classifyActivity(steps: 0), .sedentary)
        XCTAssertEqual(manager.classifyActivity(steps: 1999), .sedentary)

        // Light 2000..<5000
        XCTAssertEqual(manager.classifyActivity(steps: 2000), .light)
        XCTAssertEqual(manager.classifyActivity(steps: 4999), .light)

        // Moderate 5000..<10000
        XCTAssertEqual(manager.classifyActivity(steps: 5000), .moderate)
        XCTAssertEqual(manager.classifyActivity(steps: 9999), .moderate)

        // Active >= 10000
        XCTAssertEqual(manager.classifyActivity(steps: 10000), .active)
        XCTAssertEqual(manager.classifyActivity(steps: 15000), .active)
    }
}
