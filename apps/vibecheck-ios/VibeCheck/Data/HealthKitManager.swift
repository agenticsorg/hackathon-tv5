import Foundation
import HealthKit
import Observation

@Observable
class HealthKitManager {
    private let healthStore = HKHealthStore()

    var currentHRV: Double?
    var lastSleepHours: Double?
    var restingHeartRate: Double?
    var stepsToday: Double?
    var activityLevel: ActivityLevel = .unknown
    var isLoading = false
    var isAuthorized = false
    var errorMessage: String?

    enum ActivityLevel: String, CaseIterable {
        case sedentary = "Sedentary"
        case light = "Light"
        case moderate = "Moderate"
        case active = "Active"
        case unknown = "Unknown"
    }

    private let readTypes: Set<HKObjectType> = {
        var types = Set<HKObjectType>()
        if let hrvType = HKObjectType.quantityType(forIdentifier: .heartRateVariabilitySDNN) {
            types.insert(hrvType)
        }
        if let restingHRType = HKObjectType.quantityType(forIdentifier: .restingHeartRate) {
            types.insert(restingHRType)
        }
        if let stepsType = HKObjectType.quantityType(forIdentifier: .stepCount) {
            types.insert(stepsType)
        }
        if let activeEnergyType = HKObjectType.quantityType(forIdentifier: .activeEnergyBurned) {
            types.insert(activeEnergyType)
        }
        if let sleepType = HKObjectType.categoryType(forIdentifier: .sleepAnalysis) {
            types.insert(sleepType)
        }
        return types
    }()

    var isHealthDataAvailable: Bool {
        HKHealthStore.isHealthDataAvailable()
    }

    func requestAuthorization() async throws {
        guard isHealthDataAvailable else {
            errorMessage = "Health data not available on this device"
            return
        }

        try await healthStore.requestAuthorization(toShare: [], read: readTypes)
        isAuthorized = true
    }

    func fetchCurrentContext() async {
        isLoading = true
        defer { isLoading = false }

        await withTaskGroup(of: Void.self) { group in
            group.addTask { await self.fetchHRV() }
            group.addTask { await self.fetchSleep() }
            group.addTask { await self.fetchSteps() }
            group.addTask { await self.fetchRestingHeartRate() }
        }
    }

    // MARK: - HRV
    private func fetchHRV() async {
        guard let type = HKQuantityType.quantityType(forIdentifier: .heartRateVariabilitySDNN) else { return }

        let predicate = HKQuery.predicateForSamples(
            withStart: Date().addingTimeInterval(-3600 * 6), // last 6 hours
            end: Date()
        )

        do {
            let descriptor = HKSampleQueryDescriptor(
                predicates: [.quantitySample(type: type, predicate: predicate)],
                sortDescriptors: [SortDescriptor(\.startDate, order: .reverse)],
                limit: 1
            )

            let results = try await descriptor.result(for: healthStore)
            if let sample = results.first {
                currentHRV = sample.quantity.doubleValue(for: HKUnit.secondUnit(with: .milli))
            }
        } catch {
            print("HRV fetch error: \(error)")
        }
    }

    // MARK: - Sleep
    private func fetchSleep() async {
        guard let type = HKCategoryType.categoryType(forIdentifier: .sleepAnalysis) else { return }

        let calendar = Calendar.current
        let startOfYesterday = calendar.date(byAdding: .day, value: -1, to: calendar.startOfDay(for: Date()))!

        let predicate = HKQuery.predicateForSamples(
            withStart: startOfYesterday,
            end: Date()
        )

        do {
            let descriptor = HKSampleQueryDescriptor(
                predicates: [.categorySample(type: type, predicate: predicate)],
                sortDescriptors: [SortDescriptor(\.startDate, order: .reverse)]
            )

            let results = try await descriptor.result(for: healthStore)

            // Filter for actual asleep states (not in-bed)
            let asleepValues: Set<Int> = [
                HKCategoryValueSleepAnalysis.asleepCore.rawValue,
                HKCategoryValueSleepAnalysis.asleepDeep.rawValue,
                HKCategoryValueSleepAnalysis.asleepREM.rawValue,
                HKCategoryValueSleepAnalysis.asleepUnspecified.rawValue
            ]

            let asleepSamples = results.filter { asleepValues.contains($0.value) }

            let totalSleepSeconds = asleepSamples.reduce(0.0) { total, sample in
                total + sample.endDate.timeIntervalSince(sample.startDate)
            }

            lastSleepHours = totalSleepSeconds / 3600.0
        } catch {
            print("Sleep fetch error: \(error)")
        }
    }

    // MARK: - Steps
    private func fetchSteps() async {
        guard let type = HKQuantityType.quantityType(forIdentifier: .stepCount) else { return }

        let startOfDay = Calendar.current.startOfDay(for: Date())
        let predicate = HKQuery.predicateForSamples(withStart: startOfDay, end: Date())

        do {
            let descriptor = HKStatisticsQueryDescriptor(
                predicate: .quantitySample(type: type, predicate: predicate),
                options: .cumulativeSum
            )

            let result = try await descriptor.result(for: healthStore)
            if let sum = result?.sumQuantity() {
                let steps = sum.doubleValue(for: .count())
                stepsToday = steps
                activityLevel = classifyActivity(steps: steps)
            }
        } catch {
            print("Steps fetch error: \(error)")
        }
    }

    // MARK: - Resting Heart Rate
    private func fetchRestingHeartRate() async {
        guard let type = HKQuantityType.quantityType(forIdentifier: .restingHeartRate) else { return }

        let predicate = HKQuery.predicateForSamples(
            withStart: Date().addingTimeInterval(-86400), // last 24 hours
            end: Date()
        )

        do {
            let descriptor = HKSampleQueryDescriptor(
                predicates: [.quantitySample(type: type, predicate: predicate)],
                sortDescriptors: [SortDescriptor(\.startDate, order: .reverse)],
                limit: 1
            )

            let results = try await descriptor.result(for: healthStore)
            if let sample = results.first {
                restingHeartRate = sample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute()))
            }
        } catch {
            print("Resting HR fetch error: \(error)")
        }
    }

    // MARK: - Helpers
    func classifyActivity(steps: Double) -> ActivityLevel {
        switch steps {
        case 0..<2000:
            return .sedentary
        case 2000..<5000:
            return .light
        case 5000..<10000:
            return .moderate
        default:
            return .active
        }
    }
}
