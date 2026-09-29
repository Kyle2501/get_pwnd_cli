package main

import (
	"encoding/json"
	"fmt"
)

// GasStationBrand represents a single fuel station brand and its corporate parent.
type GasStationBrand struct {
	Brand           string `json:"brand"`
	ParentCompany   string `json:"parent_company"`
	CorporateFamily string `json:"corporate_family"` // e.g., "Refiner", "Logistics", "Retailer"
}

// brandRelationshipMap acts as an in-memory database or lookup table for corporate relationships.
// In a production system, this would likely be loaded from a database or a more complex configuration file.
var brandRelationshipMap = map[string]GasStationBrand{
	"Arco": {
		Brand:           "ARCO",
		ParentCompany:   "Marathon Petroleum (MPC)",
		CorporateFamily: "Downstream Refiner",
	},
	"Chevron": {
		Brand:           "Chevron",
		ParentCompany:   "Chevron Corporation (CVX)",
		CorporateFamily: "Integrated Major",
	},
	"Loves Travel Stop": {
		Brand:           "Love's Travel Stop",
		ParentCompany:   "Love's Travel Stops & Country Stores (Private)",
		CorporateFamily: "Logistics/Travel Center",
	},
	"Pilot": {
		Brand:           "Pilot",
		ParentCompany:   "Pilot Company (Berkshire Hathaway)",
		CorporateFamily: "Logistics/Travel Center",
	},
	"Flying J": {
		Brand:           "Flying J",
		ParentCompany:   "Pilot Company (Berkshire Hathaway)",
		CorporateFamily: "Logistics/Travel Center",
	},
	"Quiktrip": {
		Brand:           "QuikTrip (QT)",
		ParentCompany:   "QuikTrip Corporation (Private)",
		CorporateFamily: "Regional Retailer",
	},
    // ... add all other brands found in the OSM data
}

// GetBrandInfo is a function that your API would expose to query the index.
func GetBrandInfo(brandName string) (GasStationBrand, error) {
	info, ok := brandRelationshipMap[brandName]
	if !ok {
		return GasStationBrand{}, fmt.Errorf("brand '%s' not found in relationship map", brandName)
	}
	return info, nil
}

func main() {
    // Example API query from your frontend
	queryBrand := "Flying J"
	brandInfo, err := GetBrandInfo(queryBrand)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}

    // Convert the struct to JSON to serve as an API response
	jsonData, _ := json.MarshalIndent(brandInfo, "", "  ")
	fmt.Println(string(jsonData))
}

// --- SAMPLE OUTPUT ---
// {
//   "brand": "Flying J",
//   "parent_company": "Pilot Company (Berkshire Hathaway)",
//   "corporate_family": "Logistics/Travel Center"
// }
