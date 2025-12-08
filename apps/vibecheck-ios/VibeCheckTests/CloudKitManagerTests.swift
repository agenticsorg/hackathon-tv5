import XCTest
import CloudKit
@testable import VibeCheck

class CloudKitManagerTests: XCTestCase {
    
    var mockContainer: MockCKContainer!
    var mockDatabase: MockCKDatabase!
    var manager: CloudKitManager!
    
    override func setUp() {
        super.setUp()
        mockContainer = MockCKContainer()
        mockDatabase = mockContainer.mockPrivateDB
        // Dependency Injection needed for CloudKitManager
        manager = CloudKitManager(container: mockContainer)
    }
    
    func testToCKRecord() {
        // Arrange
        let timestamp = Date()
        let familyVibe = FamilyMemberVibe(
            id: UUID(),
            userID: "user_123",
            vibeKeyword: "Chill",
            vibeEnergy: "low",
            vibeStress: "relaxed",
            colorHex: "#00FF00",
            lastUpdated: timestamp
        )
        
        // Act
        let record = familyVibe.toCKRecord()
        
        // Assert
        XCTAssertEqual(record.recordType, "FamilyVibe")
        XCTAssertEqual(record["vibe_keyword"] as? String, "Chill")
        XCTAssertEqual(record["user_id"] as? String, "user_123")
        XCTAssertEqual(record["color_hex"] as? String, "#00FF00")
    }
    
    func testPublishVibe() {
        // Arrange
        let expectation = self.expectation(description: "Publish Vibe")
        let familyVibe = FamilyMemberVibe(
            id: UUID(),
            userID: "me",
            vibeKeyword: "Stressed",
            vibeEnergy: "high",
            vibeStress: "high",
            colorHex: "#FF0000",
            lastUpdated: Date()
        )
        
        // Act
        manager.publishVibe(familyVibe) { result in
            switch result {
            case .success:
                expectation.fulfill()
            case .failure(let error):
                XCTFail("Failed with error: \(error)")
            }
        }
        
        waitForExpectations(timeout: 1.0, handler: nil)
        
        // Assert
        XCTAssertEqual(mockDatabase.savedRecords.count, 1)
        XCTAssertEqual(mockDatabase.savedRecords.first?["vibe_keyword"] as? String, "Stressed")
    }
}
