// AUTO-GENERATED — do not edit by hand.
// Source: benchmarks/gerad-g2014-22/instance7/crew.csv + duties.csv
// Generator: scripts/gen_all_gerad_js.py
// Pipeline: GERAD instance7 → Roster/Duty/CrewMember → workers[]/shifts[]
//
// GERAD G-2014-22 Instance 7 (Kasirzadeh, Saddoune & Soumis 2014)
// 305 crew · 3584 duties · 761h horizon
// Normalization offset: 11h subtracted from all start_hour values.

export const GERAD_INSTANCE7_META = {
  "source": "GERAD G-2014-22 Instance 7 (Kasirzadeh, Saddoune & Soumis 2014)",
  "total_crew": 305,
  "total_duties": 3584,
  "qualifications": [
    "A319",
    "A320",
    "A321"
  ],
  "bases": [
    "BASE1",
    "BASE2",
    "BASE3"
  ],
  "horizon_hours": 761,
  "max_hours_per_worker": 80,
  "normalization_offset_hours": 11,
  "note": "start_hour values are normalized (min subtracted) so the optimizer receives relative hours rather than absolute epoch offsets. Temporal structure (gaps, overlaps, rest periods) is preserved."
};

// workers[]: each GERAD CrewMember projected to UltraCrew Worker schema.
// id: numeric crew_id (C0001 → 1), skills: [qualification]
export const GERAD_INSTANCE7_WORKERS = [
  {
    "id": 1,
    "skills": [
      "A320"
    ],
    "name": "Lea Blanc",
    "base": "BASE1",
    "gerad_id": "C0001",
    "contract_type": "full_time"
  },
  {
    "id": 2,
    "skills": [
      "A320"
    ],
    "name": "Clara Fontaine",
    "base": "BASE1",
    "gerad_id": "C0002",
    "contract_type": "part_time"
  },
  {
    "id": 3,
    "skills": [
      "A319"
    ],
    "name": "Philippe Lefebvre",
    "base": "BASE1",
    "gerad_id": "C0003",
    "contract_type": "part_time"
  },
  {
    "id": 4,
    "skills": [
      "A321"
    ],
    "name": "Maxime Roux",
    "base": "BASE1",
    "gerad_id": "C0004",
    "contract_type": "part_time"
  },
  {
    "id": 5,
    "skills": [
      "A321"
    ],
    "name": "Raphael Bernard",
    "base": "BASE1",
    "gerad_id": "C0005",
    "contract_type": "full_time"
  },
  {
    "id": 6,
    "skills": [
      "A320"
    ],
    "name": "Stephane Blanc",
    "base": "BASE1",
    "gerad_id": "C0006",
    "contract_type": "full_time"
  },
  {
    "id": 7,
    "skills": [
      "A319"
    ],
    "name": "Ines Martin",
    "base": "BASE1",
    "gerad_id": "C0007",
    "contract_type": "full_time"
  },
  {
    "id": 8,
    "skills": [
      "A319"
    ],
    "name": "Manon Bernard",
    "base": "BASE1",
    "gerad_id": "C0008",
    "contract_type": "part_time"
  },
  {
    "id": 9,
    "skills": [
      "A320"
    ],
    "name": "Pierre Faure",
    "base": "BASE1",
    "gerad_id": "C0009",
    "contract_type": "full_time"
  },
  {
    "id": 10,
    "skills": [
      "A319"
    ],
    "name": "Margot Gauthier",
    "base": "BASE1",
    "gerad_id": "C0010",
    "contract_type": "full_time"
  },
  {
    "id": 11,
    "skills": [
      "A319"
    ],
    "name": "Anais Clement",
    "base": "BASE1",
    "gerad_id": "C0011",
    "contract_type": "full_time"
  },
  {
    "id": 12,
    "skills": [
      "A321"
    ],
    "name": "Nicolas Richard",
    "base": "BASE1",
    "gerad_id": "C0012",
    "contract_type": "full_time"
  },
  {
    "id": 13,
    "skills": [
      "A319"
    ],
    "name": "Stephane Dupont",
    "base": "BASE1",
    "gerad_id": "C0013",
    "contract_type": "part_time"
  },
  {
    "id": 14,
    "skills": [
      "A320"
    ],
    "name": "Baptiste Robert",
    "base": "BASE1",
    "gerad_id": "C0014",
    "contract_type": "full_time"
  },
  {
    "id": 15,
    "skills": [
      "A319"
    ],
    "name": "Quentin Garcia",
    "base": "BASE1",
    "gerad_id": "C0015",
    "contract_type": "full_time"
  },
  {
    "id": 16,
    "skills": [
      "A320"
    ],
    "name": "Florian David",
    "base": "BASE1",
    "gerad_id": "C0016",
    "contract_type": "full_time"
  },
  {
    "id": 17,
    "skills": [
      "A320"
    ],
    "name": "Celine Petit",
    "base": "BASE1",
    "gerad_id": "C0017",
    "contract_type": "full_time"
  },
  {
    "id": 18,
    "skills": [
      "A321"
    ],
    "name": "Sebastien Thomas",
    "base": "BASE1",
    "gerad_id": "C0018",
    "contract_type": "full_time"
  },
  {
    "id": 19,
    "skills": [
      "A319"
    ],
    "name": "Camille Simon",
    "base": "BASE1",
    "gerad_id": "C0019",
    "contract_type": "full_time"
  },
  {
    "id": 20,
    "skills": [
      "A321"
    ],
    "name": "Baptiste Andre",
    "base": "BASE1",
    "gerad_id": "C0020",
    "contract_type": "full_time"
  },
  {
    "id": 21,
    "skills": [
      "A320"
    ],
    "name": "Sophie Andre",
    "base": "BASE1",
    "gerad_id": "C0021",
    "contract_type": "full_time"
  },
  {
    "id": 22,
    "skills": [
      "A320"
    ],
    "name": "Laure Legrand",
    "base": "BASE1",
    "gerad_id": "C0022",
    "contract_type": "full_time"
  },
  {
    "id": 23,
    "skills": [
      "A319"
    ],
    "name": "Clara Francois",
    "base": "BASE1",
    "gerad_id": "C0023",
    "contract_type": "full_time"
  },
  {
    "id": 24,
    "skills": [
      "A321"
    ],
    "name": "Clara Bernard",
    "base": "BASE1",
    "gerad_id": "C0024",
    "contract_type": "full_time"
  },
  {
    "id": 25,
    "skills": [
      "A321"
    ],
    "name": "Benoit Garnier",
    "base": "BASE1",
    "gerad_id": "C0025",
    "contract_type": "part_time"
  },
  {
    "id": 26,
    "skills": [
      "A319"
    ],
    "name": "Margot Clement",
    "base": "BASE1",
    "gerad_id": "C0026",
    "contract_type": "full_time"
  },
  {
    "id": 27,
    "skills": [
      "A320"
    ],
    "name": "Elise Bonnet",
    "base": "BASE1",
    "gerad_id": "C0027",
    "contract_type": "full_time"
  },
  {
    "id": 28,
    "skills": [
      "A319"
    ],
    "name": "Laurent Gauthier",
    "base": "BASE1",
    "gerad_id": "C0028",
    "contract_type": "full_time"
  },
  {
    "id": 29,
    "skills": [
      "A321"
    ],
    "name": "Margot Morin",
    "base": "BASE1",
    "gerad_id": "C0029",
    "contract_type": "full_time"
  },
  {
    "id": 30,
    "skills": [
      "A319"
    ],
    "name": "Thibault Richard",
    "base": "BASE1",
    "gerad_id": "C0030",
    "contract_type": "full_time"
  },
  {
    "id": 31,
    "skills": [
      "A319"
    ],
    "name": "Sandrine Leroy",
    "base": "BASE1",
    "gerad_id": "C0031",
    "contract_type": "full_time"
  },
  {
    "id": 32,
    "skills": [
      "A319"
    ],
    "name": "Benoit Faure",
    "base": "BASE1",
    "gerad_id": "C0032",
    "contract_type": "full_time"
  },
  {
    "id": 33,
    "skills": [
      "A320"
    ],
    "name": "Michel Lopez",
    "base": "BASE1",
    "gerad_id": "C0033",
    "contract_type": "full_time"
  },
  {
    "id": 34,
    "skills": [
      "A321"
    ],
    "name": "Julien Clement",
    "base": "BASE1",
    "gerad_id": "C0034",
    "contract_type": "full_time"
  },
  {
    "id": 35,
    "skills": [
      "A320"
    ],
    "name": "Laurent Roux",
    "base": "BASE1",
    "gerad_id": "C0035",
    "contract_type": "full_time"
  },
  {
    "id": 36,
    "skills": [
      "A319"
    ],
    "name": "Alexis Bertrand",
    "base": "BASE1",
    "gerad_id": "C0036",
    "contract_type": "full_time"
  },
  {
    "id": 37,
    "skills": [
      "A319"
    ],
    "name": "Baptiste Morin",
    "base": "BASE1",
    "gerad_id": "C0037",
    "contract_type": "part_time"
  },
  {
    "id": 38,
    "skills": [
      "A321"
    ],
    "name": "Sebastien Petit",
    "base": "BASE1",
    "gerad_id": "C0038",
    "contract_type": "full_time"
  },
  {
    "id": 39,
    "skills": [
      "A320"
    ],
    "name": "Valerie Fournier",
    "base": "BASE1",
    "gerad_id": "C0039",
    "contract_type": "part_time"
  },
  {
    "id": 40,
    "skills": [
      "A319"
    ],
    "name": "Marie Vincent",
    "base": "BASE1",
    "gerad_id": "C0040",
    "contract_type": "full_time"
  },
  {
    "id": 41,
    "skills": [
      "A321"
    ],
    "name": "Christophe Mercier",
    "base": "BASE1",
    "gerad_id": "C0041",
    "contract_type": "full_time"
  },
  {
    "id": 42,
    "skills": [
      "A321"
    ],
    "name": "Philippe Robert",
    "base": "BASE1",
    "gerad_id": "C0042",
    "contract_type": "full_time"
  },
  {
    "id": 43,
    "skills": [
      "A319"
    ],
    "name": "Pauline Schmitt",
    "base": "BASE1",
    "gerad_id": "C0043",
    "contract_type": "full_time"
  },
  {
    "id": 44,
    "skills": [
      "A321"
    ],
    "name": "Sylvie Laurent",
    "base": "BASE1",
    "gerad_id": "C0044",
    "contract_type": "full_time"
  },
  {
    "id": 45,
    "skills": [
      "A319"
    ],
    "name": "Lola Lopez",
    "base": "BASE1",
    "gerad_id": "C0045",
    "contract_type": "full_time"
  },
  {
    "id": 46,
    "skills": [
      "A320"
    ],
    "name": "Hugo Dumont",
    "base": "BASE1",
    "gerad_id": "C0046",
    "contract_type": "full_time"
  },
  {
    "id": 47,
    "skills": [
      "A321"
    ],
    "name": "Lea Renard",
    "base": "BASE1",
    "gerad_id": "C0047",
    "contract_type": "full_time"
  },
  {
    "id": 48,
    "skills": [
      "A319"
    ],
    "name": "Florian Thomas",
    "base": "BASE1",
    "gerad_id": "C0048",
    "contract_type": "part_time"
  },
  {
    "id": 49,
    "skills": [
      "A320"
    ],
    "name": "Francois Michel",
    "base": "BASE1",
    "gerad_id": "C0049",
    "contract_type": "part_time"
  },
  {
    "id": 50,
    "skills": [
      "A319"
    ],
    "name": "Nathalie Bonnet",
    "base": "BASE1",
    "gerad_id": "C0050",
    "contract_type": "part_time"
  },
  {
    "id": 51,
    "skills": [
      "A320"
    ],
    "name": "Amandine Morel",
    "base": "BASE1",
    "gerad_id": "C0051",
    "contract_type": "full_time"
  },
  {
    "id": 52,
    "skills": [
      "A319"
    ],
    "name": "Zoe Roussel",
    "base": "BASE1",
    "gerad_id": "C0052",
    "contract_type": "part_time"
  },
  {
    "id": 53,
    "skills": [
      "A319"
    ],
    "name": "Sylvie Roussel",
    "base": "BASE1",
    "gerad_id": "C0053",
    "contract_type": "full_time"
  },
  {
    "id": 54,
    "skills": [
      "A320"
    ],
    "name": "Nicolas Garnier",
    "base": "BASE1",
    "gerad_id": "C0054",
    "contract_type": "full_time"
  },
  {
    "id": 55,
    "skills": [
      "A319"
    ],
    "name": "Laurent Caron",
    "base": "BASE1",
    "gerad_id": "C0055",
    "contract_type": "full_time"
  },
  {
    "id": 56,
    "skills": [
      "A321"
    ],
    "name": "Damien Thomas",
    "base": "BASE1",
    "gerad_id": "C0056",
    "contract_type": "full_time"
  },
  {
    "id": 57,
    "skills": [
      "A321"
    ],
    "name": "Amandine Martin",
    "base": "BASE1",
    "gerad_id": "C0057",
    "contract_type": "part_time"
  },
  {
    "id": 58,
    "skills": [
      "A320"
    ],
    "name": "Benoit Renard",
    "base": "BASE1",
    "gerad_id": "C0058",
    "contract_type": "full_time"
  },
  {
    "id": 59,
    "skills": [
      "A320"
    ],
    "name": "Philippe Dumont",
    "base": "BASE1",
    "gerad_id": "C0059",
    "contract_type": "part_time"
  },
  {
    "id": 60,
    "skills": [
      "A319"
    ],
    "name": "Pauline Robin",
    "base": "BASE1",
    "gerad_id": "C0060",
    "contract_type": "full_time"
  },
  {
    "id": 61,
    "skills": [
      "A319"
    ],
    "name": "Guillaume Fournier",
    "base": "BASE1",
    "gerad_id": "C0061",
    "contract_type": "part_time"
  },
  {
    "id": 62,
    "skills": [
      "A319"
    ],
    "name": "Pierre Girard",
    "base": "BASE1",
    "gerad_id": "C0062",
    "contract_type": "full_time"
  },
  {
    "id": 63,
    "skills": [
      "A319"
    ],
    "name": "Guillaume Bernard",
    "base": "BASE1",
    "gerad_id": "C0063",
    "contract_type": "full_time"
  },
  {
    "id": 64,
    "skills": [
      "A320"
    ],
    "name": "Thibault Robert",
    "base": "BASE1",
    "gerad_id": "C0064",
    "contract_type": "part_time"
  },
  {
    "id": 65,
    "skills": [
      "A320"
    ],
    "name": "Valerie Robin",
    "base": "BASE1",
    "gerad_id": "C0065",
    "contract_type": "full_time"
  },
  {
    "id": 66,
    "skills": [
      "A321"
    ],
    "name": "Guillaume Simon",
    "base": "BASE1",
    "gerad_id": "C0066",
    "contract_type": "full_time"
  },
  {
    "id": 67,
    "skills": [
      "A321"
    ],
    "name": "Lola Dumont",
    "base": "BASE1",
    "gerad_id": "C0067",
    "contract_type": "full_time"
  },
  {
    "id": 68,
    "skills": [
      "A321"
    ],
    "name": "Laure Henry",
    "base": "BASE1",
    "gerad_id": "C0068",
    "contract_type": "full_time"
  },
  {
    "id": 69,
    "skills": [
      "A319"
    ],
    "name": "Adrien Dubois",
    "base": "BASE1",
    "gerad_id": "C0069",
    "contract_type": "full_time"
  },
  {
    "id": 70,
    "skills": [
      "A320"
    ],
    "name": "Valerie Durand",
    "base": "BASE1",
    "gerad_id": "C0070",
    "contract_type": "full_time"
  },
  {
    "id": 71,
    "skills": [
      "A320"
    ],
    "name": "Damien Collin",
    "base": "BASE1",
    "gerad_id": "C0071",
    "contract_type": "full_time"
  },
  {
    "id": 72,
    "skills": [
      "A319"
    ],
    "name": "Marie Petit",
    "base": "BASE1",
    "gerad_id": "C0072",
    "contract_type": "full_time"
  },
  {
    "id": 73,
    "skills": [
      "A321"
    ],
    "name": "Maxime Bernard",
    "base": "BASE1",
    "gerad_id": "C0073",
    "contract_type": "part_time"
  },
  {
    "id": 74,
    "skills": [
      "A321"
    ],
    "name": "Maxime Robert",
    "base": "BASE1",
    "gerad_id": "C0074",
    "contract_type": "full_time"
  },
  {
    "id": 75,
    "skills": [
      "A321"
    ],
    "name": "Thibault Garcia",
    "base": "BASE1",
    "gerad_id": "C0075",
    "contract_type": "full_time"
  },
  {
    "id": 76,
    "skills": [
      "A320"
    ],
    "name": "Jean Lopez",
    "base": "BASE1",
    "gerad_id": "C0076",
    "contract_type": "full_time"
  },
  {
    "id": 77,
    "skills": [
      "A321"
    ],
    "name": "Stephane Lefebvre",
    "base": "BASE1",
    "gerad_id": "C0077",
    "contract_type": "full_time"
  },
  {
    "id": 78,
    "skills": [
      "A320"
    ],
    "name": "Philippe Clement",
    "base": "BASE1",
    "gerad_id": "C0078",
    "contract_type": "full_time"
  },
  {
    "id": 79,
    "skills": [
      "A321"
    ],
    "name": "Lucie Giraud",
    "base": "BASE1",
    "gerad_id": "C0079",
    "contract_type": "full_time"
  },
  {
    "id": 80,
    "skills": [
      "A320"
    ],
    "name": "Julie Fournier",
    "base": "BASE1",
    "gerad_id": "C0080",
    "contract_type": "part_time"
  },
  {
    "id": 81,
    "skills": [
      "A320"
    ],
    "name": "Oceane Schmitt",
    "base": "BASE1",
    "gerad_id": "C0081",
    "contract_type": "part_time"
  },
  {
    "id": 82,
    "skills": [
      "A319"
    ],
    "name": "Zoe Dupont",
    "base": "BASE1",
    "gerad_id": "C0082",
    "contract_type": "full_time"
  },
  {
    "id": 83,
    "skills": [
      "A321"
    ],
    "name": "Lola Guerin",
    "base": "BASE1",
    "gerad_id": "C0083",
    "contract_type": "part_time"
  },
  {
    "id": 84,
    "skills": [
      "A320"
    ],
    "name": "Sandrine Henry",
    "base": "BASE1",
    "gerad_id": "C0084",
    "contract_type": "full_time"
  },
  {
    "id": 85,
    "skills": [
      "A319"
    ],
    "name": "Adrien Thomas",
    "base": "BASE1",
    "gerad_id": "C0085",
    "contract_type": "full_time"
  },
  {
    "id": 86,
    "skills": [
      "A319"
    ],
    "name": "Antoine Lefebvre",
    "base": "BASE1",
    "gerad_id": "C0086",
    "contract_type": "full_time"
  },
  {
    "id": 87,
    "skills": [
      "A320"
    ],
    "name": "Nicolas Renard",
    "base": "BASE1",
    "gerad_id": "C0087",
    "contract_type": "part_time"
  },
  {
    "id": 88,
    "skills": [
      "A320"
    ],
    "name": "Lea Masson",
    "base": "BASE1",
    "gerad_id": "C0088",
    "contract_type": "full_time"
  },
  {
    "id": 89,
    "skills": [
      "A320"
    ],
    "name": "Ines Andre",
    "base": "BASE1",
    "gerad_id": "C0089",
    "contract_type": "full_time"
  },
  {
    "id": 90,
    "skills": [
      "A319"
    ],
    "name": "Florian Perrin",
    "base": "BASE1",
    "gerad_id": "C0090",
    "contract_type": "full_time"
  },
  {
    "id": 91,
    "skills": [
      "A319"
    ],
    "name": "Marie Dumont",
    "base": "BASE1",
    "gerad_id": "C0091",
    "contract_type": "full_time"
  },
  {
    "id": 92,
    "skills": [
      "A320"
    ],
    "name": "Mathieu Morin",
    "base": "BASE1",
    "gerad_id": "C0092",
    "contract_type": "full_time"
  },
  {
    "id": 93,
    "skills": [
      "A320"
    ],
    "name": "Nathalie Robin",
    "base": "BASE1",
    "gerad_id": "C0093",
    "contract_type": "full_time"
  },
  {
    "id": 94,
    "skills": [
      "A319"
    ],
    "name": "Valerie Guerin",
    "base": "BASE1",
    "gerad_id": "C0094",
    "contract_type": "part_time"
  },
  {
    "id": 95,
    "skills": [
      "A321"
    ],
    "name": "Antoine Laurent",
    "base": "BASE1",
    "gerad_id": "C0095",
    "contract_type": "full_time"
  },
  {
    "id": 96,
    "skills": [
      "A319"
    ],
    "name": "Mathieu Lefebvre",
    "base": "BASE1",
    "gerad_id": "C0096",
    "contract_type": "full_time"
  },
  {
    "id": 97,
    "skills": [
      "A319"
    ],
    "name": "Julien Richard",
    "base": "BASE1",
    "gerad_id": "C0097",
    "contract_type": "full_time"
  },
  {
    "id": 98,
    "skills": [
      "A320"
    ],
    "name": "Zoe Muller",
    "base": "BASE1",
    "gerad_id": "C0098",
    "contract_type": "full_time"
  },
  {
    "id": 99,
    "skills": [
      "A321"
    ],
    "name": "Sophie Michel",
    "base": "BASE1",
    "gerad_id": "C0099",
    "contract_type": "full_time"
  },
  {
    "id": 100,
    "skills": [
      "A319"
    ],
    "name": "Amandine Henry",
    "base": "BASE1",
    "gerad_id": "C0100",
    "contract_type": "full_time"
  },
  {
    "id": 101,
    "skills": [
      "A321"
    ],
    "name": "Margot Masson",
    "base": "BASE1",
    "gerad_id": "C0101",
    "contract_type": "full_time"
  },
  {
    "id": 102,
    "skills": [
      "A321"
    ],
    "name": "Raphael Lambert",
    "base": "BASE1",
    "gerad_id": "C0102",
    "contract_type": "full_time"
  },
  {
    "id": 103,
    "skills": [
      "A321"
    ],
    "name": "Sylvie Dupont",
    "base": "BASE1",
    "gerad_id": "C0103",
    "contract_type": "full_time"
  },
  {
    "id": 104,
    "skills": [
      "A321"
    ],
    "name": "Celine Leroy",
    "base": "BASE1",
    "gerad_id": "C0104",
    "contract_type": "full_time"
  },
  {
    "id": 105,
    "skills": [
      "A319"
    ],
    "name": "Alexandre Garnier",
    "base": "BASE1",
    "gerad_id": "C0105",
    "contract_type": "full_time"
  },
  {
    "id": 106,
    "skills": [
      "A321"
    ],
    "name": "Alexandre Leroy",
    "base": "BASE1",
    "gerad_id": "C0106",
    "contract_type": "full_time"
  },
  {
    "id": 107,
    "skills": [
      "A320"
    ],
    "name": "Margot Thomas",
    "base": "BASE1",
    "gerad_id": "C0107",
    "contract_type": "full_time"
  },
  {
    "id": 108,
    "skills": [
      "A319"
    ],
    "name": "Valerie Collin",
    "base": "BASE1",
    "gerad_id": "C0108",
    "contract_type": "part_time"
  },
  {
    "id": 109,
    "skills": [
      "A320"
    ],
    "name": "Theo Rousseau",
    "base": "BASE1",
    "gerad_id": "C0109",
    "contract_type": "full_time"
  },
  {
    "id": 110,
    "skills": [
      "A320"
    ],
    "name": "Lea Gilles",
    "base": "BASE1",
    "gerad_id": "C0110",
    "contract_type": "full_time"
  },
  {
    "id": 111,
    "skills": [
      "A321"
    ],
    "name": "Thomas Chevalier",
    "base": "BASE1",
    "gerad_id": "C0111",
    "contract_type": "part_time"
  },
  {
    "id": 112,
    "skills": [
      "A321"
    ],
    "name": "Sandrine Fournier",
    "base": "BASE1",
    "gerad_id": "C0112",
    "contract_type": "part_time"
  },
  {
    "id": 113,
    "skills": [
      "A319"
    ],
    "name": "Laure Lopez",
    "base": "BASE1",
    "gerad_id": "C0113",
    "contract_type": "full_time"
  },
  {
    "id": 114,
    "skills": [
      "A321"
    ],
    "name": "Clement Martinez",
    "base": "BASE1",
    "gerad_id": "C0114",
    "contract_type": "full_time"
  },
  {
    "id": 115,
    "skills": [
      "A319"
    ],
    "name": "Catherine Mercier",
    "base": "BASE1",
    "gerad_id": "C0115",
    "contract_type": "part_time"
  },
  {
    "id": 116,
    "skills": [
      "A319"
    ],
    "name": "Nicolas Robin",
    "base": "BASE1",
    "gerad_id": "C0116",
    "contract_type": "full_time"
  },
  {
    "id": 117,
    "skills": [
      "A319"
    ],
    "name": "In\u00e8s Mathieu",
    "base": "BASE1",
    "gerad_id": "C0117",
    "contract_type": "full_time"
  },
  {
    "id": 118,
    "skills": [
      "A321"
    ],
    "name": "Amandine Blanc",
    "base": "BASE1",
    "gerad_id": "C0118",
    "contract_type": "full_time"
  },
  {
    "id": 119,
    "skills": [
      "A321"
    ],
    "name": "Anais Michel",
    "base": "BASE1",
    "gerad_id": "C0119",
    "contract_type": "full_time"
  },
  {
    "id": 120,
    "skills": [
      "A320"
    ],
    "name": "Thomas Martin",
    "base": "BASE1",
    "gerad_id": "C0120",
    "contract_type": "full_time"
  },
  {
    "id": 121,
    "skills": [
      "A320"
    ],
    "name": "Thomas Garcia",
    "base": "BASE1",
    "gerad_id": "C0121",
    "contract_type": "part_time"
  },
  {
    "id": 122,
    "skills": [
      "A319"
    ],
    "name": "Romain Robert",
    "base": "BASE1",
    "gerad_id": "C0122",
    "contract_type": "full_time"
  },
  {
    "id": 123,
    "skills": [
      "A319"
    ],
    "name": "Amandine Francois",
    "base": "BASE1",
    "gerad_id": "C0123",
    "contract_type": "full_time"
  },
  {
    "id": 124,
    "skills": [
      "A319"
    ],
    "name": "Clara Renard",
    "base": "BASE1",
    "gerad_id": "C0124",
    "contract_type": "part_time"
  },
  {
    "id": 125,
    "skills": [
      "A321"
    ],
    "name": "Adrien Leroy",
    "base": "BASE1",
    "gerad_id": "C0125",
    "contract_type": "part_time"
  },
  {
    "id": 126,
    "skills": [
      "A320"
    ],
    "name": "Anais Dumont",
    "base": "BASE1",
    "gerad_id": "C0126",
    "contract_type": "full_time"
  },
  {
    "id": 127,
    "skills": [
      "A321"
    ],
    "name": "Stephane David",
    "base": "BASE1",
    "gerad_id": "C0127",
    "contract_type": "full_time"
  },
  {
    "id": 128,
    "skills": [
      "A321"
    ],
    "name": "Michel Dubois",
    "base": "BASE1",
    "gerad_id": "C0128",
    "contract_type": "full_time"
  },
  {
    "id": 129,
    "skills": [
      "A319"
    ],
    "name": "Baptiste Collin",
    "base": "BASE1",
    "gerad_id": "C0129",
    "contract_type": "full_time"
  },
  {
    "id": 130,
    "skills": [
      "A320"
    ],
    "name": "Francois Caron",
    "base": "BASE1",
    "gerad_id": "C0130",
    "contract_type": "full_time"
  },
  {
    "id": 131,
    "skills": [
      "A319"
    ],
    "name": "Alexis Lefevre",
    "base": "BASE1",
    "gerad_id": "C0131",
    "contract_type": "full_time"
  },
  {
    "id": 132,
    "skills": [
      "A319"
    ],
    "name": "Emilie Garcia",
    "base": "BASE1",
    "gerad_id": "C0132",
    "contract_type": "full_time"
  },
  {
    "id": 133,
    "skills": [
      "A319"
    ],
    "name": "Clement Renard",
    "base": "BASE1",
    "gerad_id": "C0133",
    "contract_type": "full_time"
  },
  {
    "id": 134,
    "skills": [
      "A320"
    ],
    "name": "Laure Bernard",
    "base": "BASE1",
    "gerad_id": "C0134",
    "contract_type": "full_time"
  },
  {
    "id": 135,
    "skills": [
      "A321"
    ],
    "name": "Philippe Leclerc",
    "base": "BASE1",
    "gerad_id": "C0135",
    "contract_type": "full_time"
  },
  {
    "id": 136,
    "skills": [
      "A320"
    ],
    "name": "Raphael Bertrand",
    "base": "BASE1",
    "gerad_id": "C0136",
    "contract_type": "full_time"
  },
  {
    "id": 137,
    "skills": [
      "A319"
    ],
    "name": "Pierre Roux",
    "base": "BASE1",
    "gerad_id": "C0137",
    "contract_type": "full_time"
  },
  {
    "id": 138,
    "skills": [
      "A320"
    ],
    "name": "Manon Petit",
    "base": "BASE1",
    "gerad_id": "C0138",
    "contract_type": "full_time"
  },
  {
    "id": 139,
    "skills": [
      "A321"
    ],
    "name": "Manon Garnier",
    "base": "BASE1",
    "gerad_id": "C0139",
    "contract_type": "full_time"
  },
  {
    "id": 140,
    "skills": [
      "A319"
    ],
    "name": "Marie Muller",
    "base": "BASE1",
    "gerad_id": "C0140",
    "contract_type": "full_time"
  },
  {
    "id": 141,
    "skills": [
      "A319"
    ],
    "name": "In\u00e8s Martin",
    "base": "BASE1",
    "gerad_id": "C0141",
    "contract_type": "part_time"
  },
  {
    "id": 142,
    "skills": [
      "A319"
    ],
    "name": "Sylvie Chevalier",
    "base": "BASE1",
    "gerad_id": "C0142",
    "contract_type": "full_time"
  },
  {
    "id": 143,
    "skills": [
      "A321"
    ],
    "name": "Isabelle Fournier",
    "base": "BASE1",
    "gerad_id": "C0143",
    "contract_type": "full_time"
  },
  {
    "id": 144,
    "skills": [
      "A321"
    ],
    "name": "Manon Rousseau",
    "base": "BASE1",
    "gerad_id": "C0144",
    "contract_type": "part_time"
  },
  {
    "id": 145,
    "skills": [
      "A320"
    ],
    "name": "Emilie Bertrand",
    "base": "BASE1",
    "gerad_id": "C0145",
    "contract_type": "part_time"
  },
  {
    "id": 146,
    "skills": [
      "A319"
    ],
    "name": "Manon Roussel",
    "base": "BASE1",
    "gerad_id": "C0146",
    "contract_type": "full_time"
  },
  {
    "id": 147,
    "skills": [
      "A320"
    ],
    "name": "In\u00e8s Garnier",
    "base": "BASE1",
    "gerad_id": "C0147",
    "contract_type": "full_time"
  },
  {
    "id": 148,
    "skills": [
      "A321"
    ],
    "name": "Julie Michel",
    "base": "BASE1",
    "gerad_id": "C0148",
    "contract_type": "full_time"
  },
  {
    "id": 149,
    "skills": [
      "A320"
    ],
    "name": "Sandrine Martinez",
    "base": "BASE1",
    "gerad_id": "C0149",
    "contract_type": "full_time"
  },
  {
    "id": 150,
    "skills": [
      "A320"
    ],
    "name": "Adrien Renard",
    "base": "BASE1",
    "gerad_id": "C0150",
    "contract_type": "part_time"
  },
  {
    "id": 151,
    "skills": [
      "A320"
    ],
    "name": "Clara Morin",
    "base": "BASE1",
    "gerad_id": "C0151",
    "contract_type": "full_time"
  },
  {
    "id": 152,
    "skills": [
      "A321"
    ],
    "name": "Guillaume Roussel",
    "base": "BASE1",
    "gerad_id": "C0152",
    "contract_type": "full_time"
  },
  {
    "id": 153,
    "skills": [
      "A320"
    ],
    "name": "Emilie Mathieu",
    "base": "BASE1",
    "gerad_id": "C0153",
    "contract_type": "full_time"
  },
  {
    "id": 154,
    "skills": [
      "A319"
    ],
    "name": "Jade Bertrand",
    "base": "BASE1",
    "gerad_id": "C0154",
    "contract_type": "part_time"
  },
  {
    "id": 155,
    "skills": [
      "A321"
    ],
    "name": "Manon Morin",
    "base": "BASE1",
    "gerad_id": "C0155",
    "contract_type": "full_time"
  },
  {
    "id": 156,
    "skills": [
      "A321"
    ],
    "name": "Romain Bernard",
    "base": "BASE1",
    "gerad_id": "C0156",
    "contract_type": "full_time"
  },
  {
    "id": 157,
    "skills": [
      "A321"
    ],
    "name": "Marie Roussel",
    "base": "BASE1",
    "gerad_id": "C0157",
    "contract_type": "full_time"
  },
  {
    "id": 158,
    "skills": [
      "A319"
    ],
    "name": "Michel Durand",
    "base": "BASE1",
    "gerad_id": "C0158",
    "contract_type": "full_time"
  },
  {
    "id": 159,
    "skills": [
      "A319"
    ],
    "name": "Benoit Muller",
    "base": "BASE2",
    "gerad_id": "C0159",
    "contract_type": "full_time"
  },
  {
    "id": 160,
    "skills": [
      "A319"
    ],
    "name": "Alexis Garnier",
    "base": "BASE2",
    "gerad_id": "C0160",
    "contract_type": "full_time"
  },
  {
    "id": 161,
    "skills": [
      "A319"
    ],
    "name": "Guillaume Guerin",
    "base": "BASE2",
    "gerad_id": "C0161",
    "contract_type": "full_time"
  },
  {
    "id": 162,
    "skills": [
      "A321"
    ],
    "name": "Sylvie Fontaine",
    "base": "BASE2",
    "gerad_id": "C0162",
    "contract_type": "full_time"
  },
  {
    "id": 163,
    "skills": [
      "A319"
    ],
    "name": "Quentin Lambert",
    "base": "BASE2",
    "gerad_id": "C0163",
    "contract_type": "full_time"
  },
  {
    "id": 164,
    "skills": [
      "A320"
    ],
    "name": "Laure Gilles",
    "base": "BASE2",
    "gerad_id": "C0164",
    "contract_type": "full_time"
  },
  {
    "id": 165,
    "skills": [
      "A319"
    ],
    "name": "Laurent Masson",
    "base": "BASE2",
    "gerad_id": "C0165",
    "contract_type": "part_time"
  },
  {
    "id": 166,
    "skills": [
      "A321"
    ],
    "name": "Emilie Chevalier",
    "base": "BASE2",
    "gerad_id": "C0166",
    "contract_type": "full_time"
  },
  {
    "id": 167,
    "skills": [
      "A319"
    ],
    "name": "Christophe Faure",
    "base": "BASE2",
    "gerad_id": "C0167",
    "contract_type": "full_time"
  },
  {
    "id": 168,
    "skills": [
      "A320"
    ],
    "name": "Manon Robin",
    "base": "BASE2",
    "gerad_id": "C0168",
    "contract_type": "full_time"
  },
  {
    "id": 169,
    "skills": [
      "A321"
    ],
    "name": "Alexandre Chevalier",
    "base": "BASE2",
    "gerad_id": "C0169",
    "contract_type": "full_time"
  },
  {
    "id": 170,
    "skills": [
      "A321"
    ],
    "name": "Valerie Roux",
    "base": "BASE2",
    "gerad_id": "C0170",
    "contract_type": "full_time"
  },
  {
    "id": 171,
    "skills": [
      "A319"
    ],
    "name": "Aurelie Laurent",
    "base": "BASE2",
    "gerad_id": "C0171",
    "contract_type": "full_time"
  },
  {
    "id": 172,
    "skills": [
      "A319"
    ],
    "name": "In\u00e8s Mercier",
    "base": "BASE2",
    "gerad_id": "C0172",
    "contract_type": "full_time"
  },
  {
    "id": 173,
    "skills": [
      "A321"
    ],
    "name": "Lea Guerin",
    "base": "BASE2",
    "gerad_id": "C0173",
    "contract_type": "full_time"
  },
  {
    "id": 174,
    "skills": [
      "A320"
    ],
    "name": "In\u00e8s Morel",
    "base": "BASE2",
    "gerad_id": "C0174",
    "contract_type": "full_time"
  },
  {
    "id": 175,
    "skills": [
      "A319"
    ],
    "name": "Antoine Legrand",
    "base": "BASE2",
    "gerad_id": "C0175",
    "contract_type": "full_time"
  },
  {
    "id": 176,
    "skills": [
      "A320"
    ],
    "name": "Manon Mercier",
    "base": "BASE2",
    "gerad_id": "C0176",
    "contract_type": "part_time"
  },
  {
    "id": 177,
    "skills": [
      "A321"
    ],
    "name": "Lea Robert",
    "base": "BASE2",
    "gerad_id": "C0177",
    "contract_type": "full_time"
  },
  {
    "id": 178,
    "skills": [
      "A319"
    ],
    "name": "Thomas Renard",
    "base": "BASE2",
    "gerad_id": "C0178",
    "contract_type": "full_time"
  },
  {
    "id": 179,
    "skills": [
      "A320"
    ],
    "name": "Thomas Morin",
    "base": "BASE2",
    "gerad_id": "C0179",
    "contract_type": "part_time"
  },
  {
    "id": 180,
    "skills": [
      "A319"
    ],
    "name": "Francois Martin",
    "base": "BASE2",
    "gerad_id": "C0180",
    "contract_type": "full_time"
  },
  {
    "id": 181,
    "skills": [
      "A320"
    ],
    "name": "Elise Laurent",
    "base": "BASE2",
    "gerad_id": "C0181",
    "contract_type": "full_time"
  },
  {
    "id": 182,
    "skills": [
      "A319"
    ],
    "name": "Margot Andre",
    "base": "BASE2",
    "gerad_id": "C0182",
    "contract_type": "part_time"
  },
  {
    "id": 183,
    "skills": [
      "A321"
    ],
    "name": "Ines Leclerc",
    "base": "BASE2",
    "gerad_id": "C0183",
    "contract_type": "part_time"
  },
  {
    "id": 184,
    "skills": [
      "A319"
    ],
    "name": "Lucie Masson",
    "base": "BASE2",
    "gerad_id": "C0184",
    "contract_type": "part_time"
  },
  {
    "id": 185,
    "skills": [
      "A320"
    ],
    "name": "Clara Blanc",
    "base": "BASE2",
    "gerad_id": "C0185",
    "contract_type": "full_time"
  },
  {
    "id": 186,
    "skills": [
      "A321"
    ],
    "name": "Romain Thomas",
    "base": "BASE2",
    "gerad_id": "C0186",
    "contract_type": "full_time"
  },
  {
    "id": 187,
    "skills": [
      "A321"
    ],
    "name": "Quentin Petit",
    "base": "BASE2",
    "gerad_id": "C0187",
    "contract_type": "full_time"
  },
  {
    "id": 188,
    "skills": [
      "A321"
    ],
    "name": "Laurent Legrand",
    "base": "BASE2",
    "gerad_id": "C0188",
    "contract_type": "full_time"
  },
  {
    "id": 189,
    "skills": [
      "A319"
    ],
    "name": "Ines Guerin",
    "base": "BASE2",
    "gerad_id": "C0189",
    "contract_type": "full_time"
  },
  {
    "id": 190,
    "skills": [
      "A319"
    ],
    "name": "Julie Morin",
    "base": "BASE2",
    "gerad_id": "C0190",
    "contract_type": "full_time"
  },
  {
    "id": 191,
    "skills": [
      "A319"
    ],
    "name": "Stephane Masson",
    "base": "BASE2",
    "gerad_id": "C0191",
    "contract_type": "part_time"
  },
  {
    "id": 192,
    "skills": [
      "A320"
    ],
    "name": "Lucie Renard",
    "base": "BASE2",
    "gerad_id": "C0192",
    "contract_type": "full_time"
  },
  {
    "id": 193,
    "skills": [
      "A319"
    ],
    "name": "Clara Bertrand",
    "base": "BASE2",
    "gerad_id": "C0193",
    "contract_type": "full_time"
  },
  {
    "id": 194,
    "skills": [
      "A321"
    ],
    "name": "Mathieu Garnier",
    "base": "BASE2",
    "gerad_id": "C0194",
    "contract_type": "full_time"
  },
  {
    "id": 195,
    "skills": [
      "A320"
    ],
    "name": "Camille Masson",
    "base": "BASE2",
    "gerad_id": "C0195",
    "contract_type": "full_time"
  },
  {
    "id": 196,
    "skills": [
      "A320"
    ],
    "name": "Valerie Rousseau",
    "base": "BASE2",
    "gerad_id": "C0196",
    "contract_type": "full_time"
  },
  {
    "id": 197,
    "skills": [
      "A319"
    ],
    "name": "Florian Moreau",
    "base": "BASE2",
    "gerad_id": "C0197",
    "contract_type": "full_time"
  },
  {
    "id": 198,
    "skills": [
      "A319"
    ],
    "name": "Mathieu Durand",
    "base": "BASE2",
    "gerad_id": "C0198",
    "contract_type": "full_time"
  },
  {
    "id": 199,
    "skills": [
      "A321"
    ],
    "name": "Adrien Giraud",
    "base": "BASE2",
    "gerad_id": "C0199",
    "contract_type": "full_time"
  },
  {
    "id": 200,
    "skills": [
      "A321"
    ],
    "name": "Guillaume Petit",
    "base": "BASE2",
    "gerad_id": "C0200",
    "contract_type": "full_time"
  },
  {
    "id": 201,
    "skills": [
      "A321"
    ],
    "name": "Thibault Giraud",
    "base": "BASE2",
    "gerad_id": "C0201",
    "contract_type": "part_time"
  },
  {
    "id": 202,
    "skills": [
      "A320"
    ],
    "name": "Oceane Leroy",
    "base": "BASE2",
    "gerad_id": "C0202",
    "contract_type": "full_time"
  },
  {
    "id": 203,
    "skills": [
      "A319"
    ],
    "name": "Florian Bonnet",
    "base": "BASE2",
    "gerad_id": "C0203",
    "contract_type": "full_time"
  },
  {
    "id": 204,
    "skills": [
      "A320"
    ],
    "name": "Manon Masson",
    "base": "BASE2",
    "gerad_id": "C0204",
    "contract_type": "full_time"
  },
  {
    "id": 205,
    "skills": [
      "A320"
    ],
    "name": "Alexis Morin",
    "base": "BASE2",
    "gerad_id": "C0205",
    "contract_type": "part_time"
  },
  {
    "id": 206,
    "skills": [
      "A319"
    ],
    "name": "Laure Roux",
    "base": "BASE2",
    "gerad_id": "C0206",
    "contract_type": "full_time"
  },
  {
    "id": 207,
    "skills": [
      "A319"
    ],
    "name": "Sylvie Garcia",
    "base": "BASE2",
    "gerad_id": "C0207",
    "contract_type": "full_time"
  },
  {
    "id": 208,
    "skills": [
      "A321"
    ],
    "name": "Mathieu Mercier",
    "base": "BASE2",
    "gerad_id": "C0208",
    "contract_type": "full_time"
  },
  {
    "id": 209,
    "skills": [
      "A320"
    ],
    "name": "Sebastien Lopez",
    "base": "BASE2",
    "gerad_id": "C0209",
    "contract_type": "full_time"
  },
  {
    "id": 210,
    "skills": [
      "A321"
    ],
    "name": "Pierre Collin",
    "base": "BASE2",
    "gerad_id": "C0210",
    "contract_type": "full_time"
  },
  {
    "id": 211,
    "skills": [
      "A319"
    ],
    "name": "Damien Clement",
    "base": "BASE2",
    "gerad_id": "C0211",
    "contract_type": "full_time"
  },
  {
    "id": 212,
    "skills": [
      "A319"
    ],
    "name": "Anais Robert",
    "base": "BASE2",
    "gerad_id": "C0212",
    "contract_type": "full_time"
  },
  {
    "id": 213,
    "skills": [
      "A321"
    ],
    "name": "Nicolas Laurent",
    "base": "BASE2",
    "gerad_id": "C0213",
    "contract_type": "full_time"
  },
  {
    "id": 214,
    "skills": [
      "A320"
    ],
    "name": "Alexis Dupont",
    "base": "BASE2",
    "gerad_id": "C0214",
    "contract_type": "full_time"
  },
  {
    "id": 215,
    "skills": [
      "A319"
    ],
    "name": "Aurelie Garcia",
    "base": "BASE2",
    "gerad_id": "C0215",
    "contract_type": "part_time"
  },
  {
    "id": 216,
    "skills": [
      "A320"
    ],
    "name": "Lea Dubois",
    "base": "BASE2",
    "gerad_id": "C0216",
    "contract_type": "full_time"
  },
  {
    "id": 217,
    "skills": [
      "A319"
    ],
    "name": "Celine Francois",
    "base": "BASE2",
    "gerad_id": "C0217",
    "contract_type": "full_time"
  },
  {
    "id": 218,
    "skills": [
      "A319"
    ],
    "name": "Sophie Lopez",
    "base": "BASE2",
    "gerad_id": "C0218",
    "contract_type": "full_time"
  },
  {
    "id": 219,
    "skills": [
      "A319"
    ],
    "name": "Catherine Roussel",
    "base": "BASE2",
    "gerad_id": "C0219",
    "contract_type": "full_time"
  },
  {
    "id": 220,
    "skills": [
      "A320"
    ],
    "name": "Clara Schmitt",
    "base": "BASE2",
    "gerad_id": "C0220",
    "contract_type": "part_time"
  },
  {
    "id": 221,
    "skills": [
      "A319"
    ],
    "name": "Oceane Mercier",
    "base": "BASE2",
    "gerad_id": "C0221",
    "contract_type": "full_time"
  },
  {
    "id": 222,
    "skills": [
      "A320"
    ],
    "name": "Elise Nicolas",
    "base": "BASE2",
    "gerad_id": "C0222",
    "contract_type": "part_time"
  },
  {
    "id": 223,
    "skills": [
      "A319"
    ],
    "name": "Aurelie Petit",
    "base": "BASE2",
    "gerad_id": "C0223",
    "contract_type": "part_time"
  },
  {
    "id": 224,
    "skills": [
      "A321"
    ],
    "name": "Isabelle Bertrand",
    "base": "BASE2",
    "gerad_id": "C0224",
    "contract_type": "full_time"
  },
  {
    "id": 225,
    "skills": [
      "A321"
    ],
    "name": "Antoine Gilles",
    "base": "BASE2",
    "gerad_id": "C0225",
    "contract_type": "full_time"
  },
  {
    "id": 226,
    "skills": [
      "A321"
    ],
    "name": "Christophe Michel",
    "base": "BASE2",
    "gerad_id": "C0226",
    "contract_type": "full_time"
  },
  {
    "id": 227,
    "skills": [
      "A319"
    ],
    "name": "Benoit Schmitt",
    "base": "BASE2",
    "gerad_id": "C0227",
    "contract_type": "full_time"
  },
  {
    "id": 228,
    "skills": [
      "A319"
    ],
    "name": "Lola Lambert",
    "base": "BASE2",
    "gerad_id": "C0228",
    "contract_type": "full_time"
  },
  {
    "id": 229,
    "skills": [
      "A321"
    ],
    "name": "Laurent Andre",
    "base": "BASE2",
    "gerad_id": "C0229",
    "contract_type": "full_time"
  },
  {
    "id": 230,
    "skills": [
      "A320"
    ],
    "name": "Philippe Nicolas",
    "base": "BASE2",
    "gerad_id": "C0230",
    "contract_type": "part_time"
  },
  {
    "id": 231,
    "skills": [
      "A319"
    ],
    "name": "Christophe Andre",
    "base": "BASE2",
    "gerad_id": "C0231",
    "contract_type": "part_time"
  },
  {
    "id": 232,
    "skills": [
      "A319"
    ],
    "name": "Christophe Rousseau",
    "base": "BASE2",
    "gerad_id": "C0232",
    "contract_type": "full_time"
  },
  {
    "id": 233,
    "skills": [
      "A320"
    ],
    "name": "Raphael Dubois",
    "base": "BASE2",
    "gerad_id": "C0233",
    "contract_type": "full_time"
  },
  {
    "id": 234,
    "skills": [
      "A321"
    ],
    "name": "Sylvie Mathieu",
    "base": "BASE2",
    "gerad_id": "C0234",
    "contract_type": "full_time"
  },
  {
    "id": 235,
    "skills": [
      "A321"
    ],
    "name": "Elise Renard",
    "base": "BASE2",
    "gerad_id": "C0235",
    "contract_type": "full_time"
  },
  {
    "id": 236,
    "skills": [
      "A320"
    ],
    "name": "Alexis Richard",
    "base": "BASE2",
    "gerad_id": "C0236",
    "contract_type": "full_time"
  },
  {
    "id": 237,
    "skills": [
      "A320"
    ],
    "name": "Camille Schmitt",
    "base": "BASE2",
    "gerad_id": "C0237",
    "contract_type": "full_time"
  },
  {
    "id": 238,
    "skills": [
      "A320"
    ],
    "name": "Jade Dubois",
    "base": "BASE2",
    "gerad_id": "C0238",
    "contract_type": "part_time"
  },
  {
    "id": 239,
    "skills": [
      "A319"
    ],
    "name": "Chloe Michel",
    "base": "BASE2",
    "gerad_id": "C0239",
    "contract_type": "full_time"
  },
  {
    "id": 240,
    "skills": [
      "A321"
    ],
    "name": "Christophe Petit",
    "base": "BASE2",
    "gerad_id": "C0240",
    "contract_type": "full_time"
  },
  {
    "id": 241,
    "skills": [
      "A319"
    ],
    "name": "Thibault Dumont",
    "base": "BASE2",
    "gerad_id": "C0241",
    "contract_type": "full_time"
  },
  {
    "id": 242,
    "skills": [
      "A321"
    ],
    "name": "Pauline Garcia",
    "base": "BASE2",
    "gerad_id": "C0242",
    "contract_type": "full_time"
  },
  {
    "id": 243,
    "skills": [
      "A319"
    ],
    "name": "Christophe Morin",
    "base": "BASE2",
    "gerad_id": "C0243",
    "contract_type": "part_time"
  },
  {
    "id": 244,
    "skills": [
      "A319"
    ],
    "name": "Valerie Caron",
    "base": "BASE2",
    "gerad_id": "C0244",
    "contract_type": "full_time"
  },
  {
    "id": 245,
    "skills": [
      "A319"
    ],
    "name": "Raphael Fontaine",
    "base": "BASE2",
    "gerad_id": "C0245",
    "contract_type": "full_time"
  },
  {
    "id": 246,
    "skills": [
      "A319"
    ],
    "name": "Manon Schmitt",
    "base": "BASE2",
    "gerad_id": "C0246",
    "contract_type": "part_time"
  },
  {
    "id": 247,
    "skills": [
      "A320"
    ],
    "name": "Sophie Robin",
    "base": "BASE2",
    "gerad_id": "C0247",
    "contract_type": "full_time"
  },
  {
    "id": 248,
    "skills": [
      "A320"
    ],
    "name": "Isabelle Perrin",
    "base": "BASE2",
    "gerad_id": "C0248",
    "contract_type": "full_time"
  },
  {
    "id": 249,
    "skills": [
      "A321"
    ],
    "name": "Julie Dumont",
    "base": "BASE2",
    "gerad_id": "C0249",
    "contract_type": "part_time"
  },
  {
    "id": 250,
    "skills": [
      "A320"
    ],
    "name": "Michel Fontaine",
    "base": "BASE2",
    "gerad_id": "C0250",
    "contract_type": "full_time"
  },
  {
    "id": 251,
    "skills": [
      "A319"
    ],
    "name": "Maxime Thomas",
    "base": "BASE2",
    "gerad_id": "C0251",
    "contract_type": "full_time"
  },
  {
    "id": 252,
    "skills": [
      "A321"
    ],
    "name": "Manon Simon",
    "base": "BASE2",
    "gerad_id": "C0252",
    "contract_type": "full_time"
  },
  {
    "id": 253,
    "skills": [
      "A320"
    ],
    "name": "Margot Richard",
    "base": "BASE2",
    "gerad_id": "C0253",
    "contract_type": "full_time"
  },
  {
    "id": 254,
    "skills": [
      "A321"
    ],
    "name": "Pauline David",
    "base": "BASE2",
    "gerad_id": "C0254",
    "contract_type": "full_time"
  },
  {
    "id": 255,
    "skills": [
      "A319"
    ],
    "name": "Jade Dumont",
    "base": "BASE3",
    "gerad_id": "C0255",
    "contract_type": "part_time"
  },
  {
    "id": 256,
    "skills": [
      "A319"
    ],
    "name": "Margot Girard",
    "base": "BASE3",
    "gerad_id": "C0256",
    "contract_type": "full_time"
  },
  {
    "id": 257,
    "skills": [
      "A320"
    ],
    "name": "Sophie Chevalier",
    "base": "BASE3",
    "gerad_id": "C0257",
    "contract_type": "part_time"
  },
  {
    "id": 258,
    "skills": [
      "A320"
    ],
    "name": "Chloe Schmitt",
    "base": "BASE3",
    "gerad_id": "C0258",
    "contract_type": "full_time"
  },
  {
    "id": 259,
    "skills": [
      "A321"
    ],
    "name": "Julien Moreau",
    "base": "BASE3",
    "gerad_id": "C0259",
    "contract_type": "part_time"
  },
  {
    "id": 260,
    "skills": [
      "A321"
    ],
    "name": "Jean Nicolas",
    "base": "BASE3",
    "gerad_id": "C0260",
    "contract_type": "full_time"
  },
  {
    "id": 261,
    "skills": [
      "A320"
    ],
    "name": "Camille Mathieu",
    "base": "BASE3",
    "gerad_id": "C0261",
    "contract_type": "full_time"
  },
  {
    "id": 262,
    "skills": [
      "A320"
    ],
    "name": "Baptiste Durand",
    "base": "BASE3",
    "gerad_id": "C0262",
    "contract_type": "full_time"
  },
  {
    "id": 263,
    "skills": [
      "A320"
    ],
    "name": "Lola Mathieu",
    "base": "BASE3",
    "gerad_id": "C0263",
    "contract_type": "full_time"
  },
  {
    "id": 264,
    "skills": [
      "A319"
    ],
    "name": "Sebastien Andre",
    "base": "BASE3",
    "gerad_id": "C0264",
    "contract_type": "full_time"
  },
  {
    "id": 265,
    "skills": [
      "A319"
    ],
    "name": "Alexandre Morin",
    "base": "BASE3",
    "gerad_id": "C0265",
    "contract_type": "full_time"
  },
  {
    "id": 266,
    "skills": [
      "A319"
    ],
    "name": "Sophie Leroy",
    "base": "BASE3",
    "gerad_id": "C0266",
    "contract_type": "full_time"
  },
  {
    "id": 267,
    "skills": [
      "A321"
    ],
    "name": "Baptiste Clement",
    "base": "BASE3",
    "gerad_id": "C0267",
    "contract_type": "full_time"
  },
  {
    "id": 268,
    "skills": [
      "A319"
    ],
    "name": "Emilie Blanc",
    "base": "BASE3",
    "gerad_id": "C0268",
    "contract_type": "full_time"
  },
  {
    "id": 269,
    "skills": [
      "A319"
    ],
    "name": "Michel Chevalier",
    "base": "BASE3",
    "gerad_id": "C0269",
    "contract_type": "full_time"
  },
  {
    "id": 270,
    "skills": [
      "A321"
    ],
    "name": "Clara Nicolas",
    "base": "BASE3",
    "gerad_id": "C0270",
    "contract_type": "full_time"
  },
  {
    "id": 271,
    "skills": [
      "A321"
    ],
    "name": "Michel Legrand",
    "base": "BASE3",
    "gerad_id": "C0271",
    "contract_type": "full_time"
  },
  {
    "id": 272,
    "skills": [
      "A321"
    ],
    "name": "Adrien Bernard",
    "base": "BASE3",
    "gerad_id": "C0272",
    "contract_type": "full_time"
  },
  {
    "id": 273,
    "skills": [
      "A319"
    ],
    "name": "Lola Bertrand",
    "base": "BASE3",
    "gerad_id": "C0273",
    "contract_type": "full_time"
  },
  {
    "id": 274,
    "skills": [
      "A319"
    ],
    "name": "Adrien David",
    "base": "BASE3",
    "gerad_id": "C0274",
    "contract_type": "full_time"
  },
  {
    "id": 275,
    "skills": [
      "A319"
    ],
    "name": "Laurent Lefevre",
    "base": "BASE3",
    "gerad_id": "C0275",
    "contract_type": "full_time"
  },
  {
    "id": 276,
    "skills": [
      "A321"
    ],
    "name": "Laure Collin",
    "base": "BASE3",
    "gerad_id": "C0276",
    "contract_type": "part_time"
  },
  {
    "id": 277,
    "skills": [
      "A320"
    ],
    "name": "Florian Legrand",
    "base": "BASE3",
    "gerad_id": "C0277",
    "contract_type": "full_time"
  },
  {
    "id": 278,
    "skills": [
      "A321"
    ],
    "name": "Elise Schmitt",
    "base": "BASE3",
    "gerad_id": "C0278",
    "contract_type": "part_time"
  },
  {
    "id": 279,
    "skills": [
      "A321"
    ],
    "name": "Pierre Petit",
    "base": "BASE3",
    "gerad_id": "C0279",
    "contract_type": "full_time"
  },
  {
    "id": 280,
    "skills": [
      "A321"
    ],
    "name": "Laurent Lambert",
    "base": "BASE3",
    "gerad_id": "C0280",
    "contract_type": "full_time"
  },
  {
    "id": 281,
    "skills": [
      "A321"
    ],
    "name": "Anais Chevalier",
    "base": "BASE3",
    "gerad_id": "C0281",
    "contract_type": "full_time"
  },
  {
    "id": 282,
    "skills": [
      "A320"
    ],
    "name": "Zoe Francois",
    "base": "BASE3",
    "gerad_id": "C0282",
    "contract_type": "full_time"
  },
  {
    "id": 283,
    "skills": [
      "A319"
    ],
    "name": "Valerie Thomas",
    "base": "BASE3",
    "gerad_id": "C0283",
    "contract_type": "full_time"
  },
  {
    "id": 284,
    "skills": [
      "A319"
    ],
    "name": "Guillaume Lefebvre",
    "base": "BASE3",
    "gerad_id": "C0284",
    "contract_type": "full_time"
  },
  {
    "id": 285,
    "skills": [
      "A320"
    ],
    "name": "Michel Martin",
    "base": "BASE3",
    "gerad_id": "C0285",
    "contract_type": "full_time"
  },
  {
    "id": 286,
    "skills": [
      "A319"
    ],
    "name": "Florian Mathieu",
    "base": "BASE3",
    "gerad_id": "C0286",
    "contract_type": "part_time"
  },
  {
    "id": 287,
    "skills": [
      "A320"
    ],
    "name": "Baptiste Schmitt",
    "base": "BASE3",
    "gerad_id": "C0287",
    "contract_type": "part_time"
  },
  {
    "id": 288,
    "skills": [
      "A321"
    ],
    "name": "Philippe Bonnet",
    "base": "BASE3",
    "gerad_id": "C0288",
    "contract_type": "full_time"
  },
  {
    "id": 289,
    "skills": [
      "A320"
    ],
    "name": "Hugo Blanc",
    "base": "BASE3",
    "gerad_id": "C0289",
    "contract_type": "part_time"
  },
  {
    "id": 290,
    "skills": [
      "A320"
    ],
    "name": "Lea Fontaine",
    "base": "BASE3",
    "gerad_id": "C0290",
    "contract_type": "full_time"
  },
  {
    "id": 291,
    "skills": [
      "A319"
    ],
    "name": "Antoine Richard",
    "base": "BASE3",
    "gerad_id": "C0291",
    "contract_type": "part_time"
  },
  {
    "id": 292,
    "skills": [
      "A319"
    ],
    "name": "Amandine Petit",
    "base": "BASE3",
    "gerad_id": "C0292",
    "contract_type": "full_time"
  },
  {
    "id": 293,
    "skills": [
      "A319"
    ],
    "name": "Pierre Andre",
    "base": "BASE3",
    "gerad_id": "C0293",
    "contract_type": "full_time"
  },
  {
    "id": 294,
    "skills": [
      "A320"
    ],
    "name": "Manon Dupont",
    "base": "BASE3",
    "gerad_id": "C0294",
    "contract_type": "part_time"
  },
  {
    "id": 295,
    "skills": [
      "A321"
    ],
    "name": "Jade Simon",
    "base": "BASE3",
    "gerad_id": "C0295",
    "contract_type": "full_time"
  },
  {
    "id": 296,
    "skills": [
      "A320"
    ],
    "name": "Julien Simon",
    "base": "BASE3",
    "gerad_id": "C0296",
    "contract_type": "part_time"
  },
  {
    "id": 297,
    "skills": [
      "A320"
    ],
    "name": "Thomas Dumont",
    "base": "BASE3",
    "gerad_id": "C0297",
    "contract_type": "part_time"
  },
  {
    "id": 298,
    "skills": [
      "A321"
    ],
    "name": "Catherine Morel",
    "base": "BASE3",
    "gerad_id": "C0298",
    "contract_type": "full_time"
  },
  {
    "id": 299,
    "skills": [
      "A319"
    ],
    "name": "Romain Nicolas",
    "base": "BASE3",
    "gerad_id": "C0299",
    "contract_type": "full_time"
  },
  {
    "id": 300,
    "skills": [
      "A320"
    ],
    "name": "Lola Vincent",
    "base": "BASE3",
    "gerad_id": "C0300",
    "contract_type": "full_time"
  },
  {
    "id": 301,
    "skills": [
      "A319"
    ],
    "name": "Amandine Simon",
    "base": "BASE3",
    "gerad_id": "C0301",
    "contract_type": "part_time"
  },
  {
    "id": 302,
    "skills": [
      "A321"
    ],
    "name": "Raphael Renard",
    "base": "BASE3",
    "gerad_id": "C0302",
    "contract_type": "full_time"
  },
  {
    "id": 303,
    "skills": [
      "A320"
    ],
    "name": "Jean Moreau",
    "base": "BASE3",
    "gerad_id": "C0303",
    "contract_type": "full_time"
  },
  {
    "id": 304,
    "skills": [
      "A321"
    ],
    "name": "Lucie Andre",
    "base": "BASE3",
    "gerad_id": "C0304",
    "contract_type": "full_time"
  },
  {
    "id": 305,
    "skills": [
      "A321"
    ],
    "name": "Pauline Leclerc",
    "base": "BASE3",
    "gerad_id": "C0305",
    "contract_type": "full_time"
  }
];

// shifts[]: each GERAD Duty projected to UltraCrew Shift schema.
// id: numeric duty_id, start_hour: normalized FDP report time,
// duration_hours: FDP length (release - report), required_skill: crew qualification.
export const GERAD_INSTANCE7_SHIFTS = [
  {
    "id": 1,
    "start_hour": 681,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0001",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_29_19"
  },
  {
    "id": 2,
    "start_hour": 684,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0002",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_30_259,LEG_30_253,LEG_30_0"
  },
  {
    "id": 3,
    "start_hour": 708,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0003",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_31_124,LEG_31_241,LEG_31_219,LEG_31_131"
  },
  {
    "id": 4,
    "start_hour": 682,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0004",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_29_182"
  },
  {
    "id": 5,
    "start_hour": 684,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0005",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_30_236,LEG_30_212,LEG_30_209"
  },
  {
    "id": 6,
    "start_hour": 709,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D0006",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_31_222,LEG_31_123,LEG_31_129,LEG_31_28"
  },
  {
    "id": 7,
    "start_hour": 682,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0007",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_29_80"
  },
  {
    "id": 8,
    "start_hour": 684,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D0008",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_30_239,LEG_30_174,LEG_30_170,LEG_30_112"
  },
  {
    "id": 9,
    "start_hour": 663,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0009",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_29_26,LEG_29_52,LEG_29_224"
  },
  {
    "id": 10,
    "start_hour": 699,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0010",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_30_37,LEG_30_106,LEG_30_99"
  },
  {
    "id": 11,
    "start_hour": 684,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0011",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_29_93"
  },
  {
    "id": 12,
    "start_hour": 708,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0012",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_30_29"
  },
  {
    "id": 13,
    "start_hour": 712,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0013",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_31_177,LEG_31_33"
  },
  {
    "id": 14,
    "start_hour": 663,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0014",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_29_147"
  },
  {
    "id": 15,
    "start_hour": 699,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0015",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_30_151,LEG_30_60"
  },
  {
    "id": 16,
    "start_hour": 723,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0016",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_31_56"
  },
  {
    "id": 17,
    "start_hour": 661,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0017",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_29_73,LEG_29_46"
  },
  {
    "id": 18,
    "start_hour": 687,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0018",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_30_200,LEG_30_89,LEG_30_264"
  },
  {
    "id": 19,
    "start_hour": 729,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0019",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_31_72"
  },
  {
    "id": 20,
    "start_hour": 672,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0020",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_29_256,LEG_29_199"
  },
  {
    "id": 21,
    "start_hour": 693,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0021",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_30_261,LEG_30_132,LEG_30_128"
  },
  {
    "id": 22,
    "start_hour": 681,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0022",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_29_75"
  },
  {
    "id": 23,
    "start_hour": 686,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0023",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_30_70"
  },
  {
    "id": 24,
    "start_hour": 681,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0024",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_29_3"
  },
  {
    "id": 25,
    "start_hour": 685,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0025",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_30_1"
  },
  {
    "id": 26,
    "start_hour": 680,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0026",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_29_57,LEG_29_112"
  },
  {
    "id": 27,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0027",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_29_8,LEG_29_6"
  },
  {
    "id": 28,
    "start_hour": 674,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0028",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_29_248,LEG_29_2"
  },
  {
    "id": 29,
    "start_hour": 680,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0029",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_29_151"
  },
  {
    "id": 30,
    "start_hour": 684,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0030",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_30_149"
  },
  {
    "id": 31,
    "start_hour": 676,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0031",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_29_83,LEG_29_88"
  },
  {
    "id": 32,
    "start_hour": 681,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0032",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_29_179"
  },
  {
    "id": 33,
    "start_hour": 685,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0033",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_30_179"
  },
  {
    "id": 34,
    "start_hour": 678,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0034",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_29_22,LEG_29_23"
  },
  {
    "id": 35,
    "start_hour": 678,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0035",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_29_154,LEG_29_156"
  },
  {
    "id": 36,
    "start_hour": 676,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0036",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_29_25,LEG_29_18"
  },
  {
    "id": 37,
    "start_hour": 673,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0037",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_29_267,LEG_29_181"
  },
  {
    "id": 38,
    "start_hour": 665,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0038",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_29_111,LEG_29_168"
  },
  {
    "id": 39,
    "start_hour": 673,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0039",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_29_263,LEG_29_264"
  },
  {
    "id": 40,
    "start_hour": 674,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0040",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_29_140,LEG_29_141"
  },
  {
    "id": 41,
    "start_hour": 664,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0041",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_29_155,LEG_29_133"
  },
  {
    "id": 42,
    "start_hour": 663,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0042",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_29_146,LEG_29_81"
  },
  {
    "id": 43,
    "start_hour": 673,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0043",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_29_113,LEG_29_74"
  },
  {
    "id": 44,
    "start_hour": 680,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0044",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_29_170,LEG_29_251"
  },
  {
    "id": 45,
    "start_hour": 687,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0045",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_30_225"
  },
  {
    "id": 46,
    "start_hour": 722,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0046",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_31_228,LEG_31_95,LEG_31_130"
  },
  {
    "id": 47,
    "start_hour": 676,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0047",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_29_169,LEG_29_176"
  },
  {
    "id": 48,
    "start_hour": 685,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0048",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_30_260,LEG_30_7,LEG_30_135,LEG_30_138"
  },
  {
    "id": 49,
    "start_hour": 680,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0049",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_29_161,LEG_29_238"
  },
  {
    "id": 50,
    "start_hour": 687,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0050",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_30_238,LEG_30_62,LEG_30_268"
  },
  {
    "id": 51,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0051",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_31_254,LEG_31_0"
  },
  {
    "id": 52,
    "start_hour": 684,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0052",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_29_86"
  },
  {
    "id": 53,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0053",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_30_85,LEG_30_249"
  },
  {
    "id": 54,
    "start_hour": 720,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0054",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_31_82,LEG_31_169,LEG_31_176"
  },
  {
    "id": 55,
    "start_hour": 663,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0055",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_29_27,LEG_29_101"
  },
  {
    "id": 56,
    "start_hour": 685,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0056",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_30_71,LEG_30_103,LEG_30_214,LEG_30_145"
  },
  {
    "id": 57,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0057",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_29_58,LEG_29_61"
  },
  {
    "id": 58,
    "start_hour": 685,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D0058",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_30_5,LEG_30_67"
  },
  {
    "id": 59,
    "start_hour": 708,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D0059",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_31_78,LEG_31_96"
  },
  {
    "id": 60,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0060",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_29_84,LEG_29_227"
  },
  {
    "id": 61,
    "start_hour": 687,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0061",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_30_98,LEG_30_267,LEG_30_165,LEG_30_163"
  },
  {
    "id": 62,
    "start_hour": 338,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0062",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_15_205,LEG_15_204"
  },
  {
    "id": 63,
    "start_hour": 338,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0063",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_15_254,LEG_15_257"
  },
  {
    "id": 64,
    "start_hour": 327,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0064",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_15_206,LEG_15_103"
  },
  {
    "id": 65,
    "start_hour": 349,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0065",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_16_191,LEG_16_124"
  },
  {
    "id": 66,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0066",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_15_123,LEG_15_120"
  },
  {
    "id": 67,
    "start_hour": 346,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0067",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_15_256"
  },
  {
    "id": 68,
    "start_hour": 349,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0068",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_16_48,LEG_16_50,LEG_16_82"
  },
  {
    "id": 69,
    "start_hour": 386,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0069",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_17_122"
  },
  {
    "id": 70,
    "start_hour": 326,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0070",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_15_58,LEG_15_95"
  },
  {
    "id": 71,
    "start_hour": 352,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0071",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_16_2,LEG_16_61"
  },
  {
    "id": 72,
    "start_hour": 373,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0072",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_17_191,LEG_17_124"
  },
  {
    "id": 73,
    "start_hour": 347,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0073",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_15_128"
  },
  {
    "id": 74,
    "start_hour": 367,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0074",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_16_76"
  },
  {
    "id": 75,
    "start_hour": 372,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0075",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_17_9,LEG_17_153,LEG_17_151"
  },
  {
    "id": 76,
    "start_hour": 408,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0076",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_18_158,LEG_18_144,LEG_18_203"
  },
  {
    "id": 77,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0077",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_15_202,LEG_15_148"
  },
  {
    "id": 78,
    "start_hour": 350,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0078",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_16_176,LEG_16_178"
  },
  {
    "id": 79,
    "start_hour": 372,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0079",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_17_166,LEG_17_185"
  },
  {
    "id": 80,
    "start_hour": 398,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0080",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_18_101,LEG_18_94,LEG_18_171,LEG_18_199"
  },
  {
    "id": 81,
    "start_hour": 121,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0081",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_06_156,LEG_06_157"
  },
  {
    "id": 82,
    "start_hour": 108,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0082",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_06_64,LEG_06_61"
  },
  {
    "id": 83,
    "start_hour": 120,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0083",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_06_235,LEG_06_236"
  },
  {
    "id": 84,
    "start_hour": 120,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0084",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_06_125,LEG_06_127"
  },
  {
    "id": 85,
    "start_hour": 111,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0085",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_06_138,LEG_06_169"
  },
  {
    "id": 86,
    "start_hour": 123,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0086",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_06_87,LEG_06_90"
  },
  {
    "id": 87,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0087",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_06_23,LEG_06_25"
  },
  {
    "id": 88,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0088",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_06_13,LEG_06_19"
  },
  {
    "id": 89,
    "start_hour": 119,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0089",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_06_239,LEG_06_51"
  },
  {
    "id": 90,
    "start_hour": 140,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0090",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_07_86,LEG_07_130,LEG_07_137"
  },
  {
    "id": 91,
    "start_hour": 110,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0091",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_06_92,LEG_06_85,LEG_06_5"
  },
  {
    "id": 92,
    "start_hour": 146,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0092",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_07_5"
  },
  {
    "id": 93,
    "start_hour": 125,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0093",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_06_141,LEG_06_135,LEG_06_91"
  },
  {
    "id": 94,
    "start_hour": 143,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0094",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_07_218,LEG_07_216,LEG_07_143"
  },
  {
    "id": 95,
    "start_hour": 124,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0095",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_06_67,LEG_06_107"
  },
  {
    "id": 96,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0096",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_07_124,LEG_07_76"
  },
  {
    "id": 97,
    "start_hour": 125,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0097",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_06_15,LEG_06_21"
  },
  {
    "id": 98,
    "start_hour": 127,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0098",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_06_133,LEG_06_137"
  },
  {
    "id": 99,
    "start_hour": 124,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0099",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_06_83,LEG_06_82"
  },
  {
    "id": 100,
    "start_hour": 128,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0100",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_06_234,LEG_06_1"
  },
  {
    "id": 101,
    "start_hour": 110,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0101",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_06_20,LEG_06_57"
  },
  {
    "id": 102,
    "start_hour": 133,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0102",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_07_49,LEG_07_51,LEG_07_64"
  },
  {
    "id": 103,
    "start_hour": 170,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0103",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_08_17"
  },
  {
    "id": 104,
    "start_hour": 108,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0104",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_06_24,LEG_06_219"
  },
  {
    "id": 105,
    "start_hour": 133,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0105",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_07_195,LEG_07_125"
  },
  {
    "id": 106,
    "start_hour": 158,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0106",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_08_59,LEG_08_96"
  },
  {
    "id": 107,
    "start_hour": 131,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0107",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_06_27"
  },
  {
    "id": 108,
    "start_hour": 155,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0108",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_07_37"
  },
  {
    "id": 109,
    "start_hour": 130,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0109",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_06_158,LEG_06_187"
  },
  {
    "id": 110,
    "start_hour": 134,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0110",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_07_69,LEG_07_87"
  },
  {
    "id": 111,
    "start_hour": 176,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0111",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_08_89,LEG_08_75"
  },
  {
    "id": 112,
    "start_hour": 193,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0112",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_09_18,LEG_09_139,LEG_09_237"
  },
  {
    "id": 113,
    "start_hour": 138,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0113",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_07_40,LEG_07_197"
  },
  {
    "id": 114,
    "start_hour": 176,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0114",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_08_194"
  },
  {
    "id": 115,
    "start_hour": 181,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D0115",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_09_14,LEG_09_176,LEG_09_131,LEG_09_134"
  },
  {
    "id": 116,
    "start_hour": 125,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0116",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_06_124,LEG_06_123"
  },
  {
    "id": 117,
    "start_hour": 132,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D0117",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_07_29,LEG_07_68,LEG_07_67"
  },
  {
    "id": 118,
    "start_hour": 172,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0118",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_08_21,LEG_08_133"
  },
  {
    "id": 119,
    "start_hour": 181,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0119",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_09_132"
  },
  {
    "id": 120,
    "start_hour": 123,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0120",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_06_171,LEG_06_41,LEG_06_26"
  },
  {
    "id": 121,
    "start_hour": 132,
    "duration_hours": 27,
    "required_skill": "A321",
    "gerad_duty_id": "D0121",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_07_88,LEG_07_247,LEG_07_172,LEG_07_80,LEG_07_82"
  },
  {
    "id": 122,
    "start_hour": 171,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0122",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_08_198,LEG_08_201,LEG_08_105"
  },
  {
    "id": 123,
    "start_hour": 129,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0123",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_06_147,LEG_06_203"
  },
  {
    "id": 124,
    "start_hour": 134,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0124",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_07_102,LEG_07_95,LEG_07_256"
  },
  {
    "id": 125,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0125",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_08_158,LEG_08_73,LEG_08_160"
  },
  {
    "id": 126,
    "start_hour": 110,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0126",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_06_227,LEG_06_224"
  },
  {
    "id": 127,
    "start_hour": 132,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0127",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_07_150,LEG_07_205,LEG_07_199,LEG_07_196,LEG_07_47"
  },
  {
    "id": 128,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0128",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_08_167,LEG_08_4,LEG_08_135"
  },
  {
    "id": 129,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0129",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_06_204,LEG_06_120"
  },
  {
    "id": 130,
    "start_hour": 134,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0130",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_07_180,LEG_07_182"
  },
  {
    "id": 131,
    "start_hour": 156,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0131",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_08_150,LEG_08_204,LEG_08_205"
  },
  {
    "id": 132,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0132",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_09_72,LEG_09_242,LEG_09_35"
  },
  {
    "id": 133,
    "start_hour": 125,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0133",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_06_152,LEG_06_153"
  },
  {
    "id": 134,
    "start_hour": 132,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0134",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_07_9,LEG_07_157,LEG_07_146"
  },
  {
    "id": 135,
    "start_hour": 173,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0135",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_08_212,LEG_08_128"
  },
  {
    "id": 136,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0136",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_09_202,LEG_09_201,LEG_09_105"
  },
  {
    "id": 137,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0137",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_19_187,LEG_19_186"
  },
  {
    "id": 138,
    "start_hour": 422,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0138",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_19_112,LEG_19_116"
  },
  {
    "id": 139,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0139",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_19_227,LEG_19_230"
  },
  {
    "id": 140,
    "start_hour": 434,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0140",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_19_220,LEG_19_219"
  },
  {
    "id": 141,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0141",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_19_114,LEG_19_111"
  },
  {
    "id": 142,
    "start_hour": 422,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0142",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_19_51,LEG_19_86"
  },
  {
    "id": 143,
    "start_hour": 446,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0143",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_20_215"
  },
  {
    "id": 144,
    "start_hour": 482,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0144",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_21_235,LEG_21_19,LEG_21_197"
  },
  {
    "id": 145,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0145",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_19_184,LEG_19_138"
  },
  {
    "id": 146,
    "start_hour": 446,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0146",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_20_162,LEG_20_164"
  },
  {
    "id": 147,
    "start_hour": 468,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0147",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_21_168,LEG_21_187"
  },
  {
    "id": 148,
    "start_hour": 494,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0148",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_22_101,LEG_22_94,LEG_22_19,LEG_22_196"
  },
  {
    "id": 149,
    "start_hour": 2,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0149",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_01_222,LEG_01_225"
  },
  {
    "id": 150,
    "start_hour": 2,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0150",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_01_177,LEG_01_176"
  },
  {
    "id": 151,
    "start_hour": 6,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0151",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_01_173"
  },
  {
    "id": 152,
    "start_hour": 26,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0152",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_02_123"
  },
  {
    "id": 153,
    "start_hour": 8,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0153",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_01_73,LEG_01_201"
  },
  {
    "id": 154,
    "start_hour": 24,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0154",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_02_239,LEG_02_148,LEG_02_207"
  },
  {
    "id": 155,
    "start_hour": 9,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0155",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_01_45"
  },
  {
    "id": 156,
    "start_hour": 6,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0156",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_01_2,LEG_01_171"
  },
  {
    "id": 157,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0157",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_01_167,LEG_01_164,LEG_01_37"
  },
  {
    "id": 158,
    "start_hour": 25,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0158",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_02_167,LEG_02_155"
  },
  {
    "id": 159,
    "start_hour": 48,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0159",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_03_162,LEG_03_148,LEG_03_207"
  },
  {
    "id": 160,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0160",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_01_169,LEG_01_90"
  },
  {
    "id": 161,
    "start_hour": 15,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0161",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_02_136,LEG_02_228,LEG_02_83"
  },
  {
    "id": 162,
    "start_hour": 50,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0162",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_03_123"
  },
  {
    "id": 163,
    "start_hour": 2,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0163",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_01_214"
  },
  {
    "id": 164,
    "start_hour": 18,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0164",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_02_56,LEG_02_233,LEG_02_118"
  },
  {
    "id": 165,
    "start_hour": 37,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0165",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_03_195,LEG_03_125"
  },
  {
    "id": 166,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0166",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_01_104,LEG_01_60"
  },
  {
    "id": 167,
    "start_hour": 28,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0167",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_02_77,LEG_02_121"
  },
  {
    "id": 168,
    "start_hour": 10,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0168",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_01_224"
  },
  {
    "id": 169,
    "start_hour": 13,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0169",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_02_49,LEG_02_51,LEG_02_65"
  },
  {
    "id": 170,
    "start_hour": 50,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0170",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_03_17,LEG_03_184,LEG_03_203"
  },
  {
    "id": 171,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0171",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_01_109"
  },
  {
    "id": 172,
    "start_hour": 35,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0172",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_02_79"
  },
  {
    "id": 173,
    "start_hour": 53,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0173",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_03_57,LEG_03_247"
  },
  {
    "id": 174,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0174",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_01_174,LEG_01_126"
  },
  {
    "id": 175,
    "start_hour": 14,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0175",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_02_180,LEG_02_182"
  },
  {
    "id": 176,
    "start_hour": 36,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0176",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_03_170,LEG_03_189"
  },
  {
    "id": 177,
    "start_hour": 62,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0177",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_04_161,LEG_04_164,LEG_04_20,LEG_04_200"
  },
  {
    "id": 178,
    "start_hour": 10,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0178",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_01_206"
  },
  {
    "id": 179,
    "start_hour": 13,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0179",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_02_113,LEG_02_105"
  },
  {
    "id": 180,
    "start_hour": 39,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0180",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_03_136,LEG_03_228,LEG_03_213,LEG_03_85"
  },
  {
    "id": 181,
    "start_hour": 77,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0181",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_04_57,LEG_04_247"
  },
  {
    "id": 182,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0182",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_01_178,LEG_01_7"
  },
  {
    "id": 183,
    "start_hour": 25,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0183",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_02_45,LEG_02_171,LEG_02_55"
  },
  {
    "id": 184,
    "start_hour": 54,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0184",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_03_3,LEG_03_7"
  },
  {
    "id": 185,
    "start_hour": 74,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0185",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_04_5,LEG_04_184,LEG_04_203"
  },
  {
    "id": 186,
    "start_hour": 632,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0186",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_27_197"
  },
  {
    "id": 187,
    "start_hour": 632,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0187",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_27_195"
  },
  {
    "id": 188,
    "start_hour": 624,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0188",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_27_190,LEG_27_84"
  },
  {
    "id": 189,
    "start_hour": 631,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0189",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_27_83,LEG_27_77"
  },
  {
    "id": 190,
    "start_hour": 640,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0190",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_28_121,LEG_28_219,LEG_28_36,LEG_28_50"
  },
  {
    "id": 191,
    "start_hour": 630,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0191",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_27_35,LEG_27_37"
  },
  {
    "id": 192,
    "start_hour": 638,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0192",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_28_229,LEG_28_135,LEG_28_228,LEG_28_222"
  },
  {
    "id": 193,
    "start_hour": 626,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0193",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_27_194"
  },
  {
    "id": 194,
    "start_hour": 626,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0194",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_27_196"
  },
  {
    "id": 195,
    "start_hour": 631,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0195",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_27_177,LEG_27_62"
  },
  {
    "id": 196,
    "start_hour": 637,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0196",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_28_221,LEG_28_122,LEG_28_95,LEG_28_129"
  },
  {
    "id": 197,
    "start_hour": 660,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0197",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_29_124,LEG_29_241,LEG_29_219,LEG_29_225"
  },
  {
    "id": 198,
    "start_hour": 629,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0198",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_27_172,LEG_27_97"
  },
  {
    "id": 199,
    "start_hour": 637,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D0199",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_28_5,LEG_28_112"
  },
  {
    "id": 200,
    "start_hour": 664,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0200",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_29_177"
  },
  {
    "id": 201,
    "start_hour": 634,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0201",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_27_81"
  },
  {
    "id": 202,
    "start_hour": 636,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D0202",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_28_40,LEG_28_91,LEG_28_89,LEG_28_262"
  },
  {
    "id": 203,
    "start_hour": 681,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0203",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_29_72"
  },
  {
    "id": 204,
    "start_hour": 685,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0204",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_30_87,LEG_30_30,LEG_30_228,LEG_30_222"
  },
  {
    "id": 205,
    "start_hour": 687,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0205",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_30_45,LEG_30_205"
  },
  {
    "id": 206,
    "start_hour": 699,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0206",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_30_254,LEG_30_257"
  },
  {
    "id": 207,
    "start_hour": 699,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0207",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_30_116,LEG_30_115"
  },
  {
    "id": 208,
    "start_hour": 688,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0208",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_30_16,LEG_30_245"
  },
  {
    "id": 209,
    "start_hour": 688,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0209",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_30_191,LEG_30_12"
  },
  {
    "id": 210,
    "start_hour": 708,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0210",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_30_202"
  },
  {
    "id": 211,
    "start_hour": 726,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0211",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_31_205"
  },
  {
    "id": 212,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0212",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_30_13,LEG_30_11"
  },
  {
    "id": 213,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0213",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_30_194,LEG_30_195"
  },
  {
    "id": 214,
    "start_hour": 699,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0214",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_30_206"
  },
  {
    "id": 215,
    "start_hour": 703,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0215",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_30_197,LEG_30_147"
  },
  {
    "id": 216,
    "start_hour": 711,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D0216",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_31_98,LEG_31_268,LEG_31_269"
  },
  {
    "id": 217,
    "start_hour": 703,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0217",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_30_188"
  },
  {
    "id": 218,
    "start_hour": 710,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0218",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_31_59,LEG_31_63,LEG_31_11"
  },
  {
    "id": 219,
    "start_hour": 538,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0219",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_23_237"
  },
  {
    "id": 220,
    "start_hour": 541,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0220",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_24_48,LEG_24_50,LEG_24_54"
  },
  {
    "id": 221,
    "start_hour": 582,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0221",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_25_3,LEG_25_7"
  },
  {
    "id": 222,
    "start_hour": 602,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0222",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_26_5,LEG_26_164,LEG_26_181"
  },
  {
    "id": 223,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0223",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_23_202,LEG_23_148"
  },
  {
    "id": 224,
    "start_hour": 542,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D0224",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_24_105,LEG_24_161,LEG_24_162"
  },
  {
    "id": 225,
    "start_hour": 575,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0225",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_25_14,LEG_25_173"
  },
  {
    "id": 226,
    "start_hour": 600,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0226",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_26_159,LEG_26_131,LEG_26_185"
  },
  {
    "id": 227,
    "start_hour": 539,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0227",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_23_128"
  },
  {
    "id": 228,
    "start_hour": 559,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0228",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_24_76"
  },
  {
    "id": 229,
    "start_hour": 564,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0229",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_25_87,LEG_25_30"
  },
  {
    "id": 230,
    "start_hour": 588,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0230",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_26_67,LEG_26_114"
  },
  {
    "id": 231,
    "start_hour": 538,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0231",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_23_256"
  },
  {
    "id": 232,
    "start_hour": 541,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0232",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_24_217,LEG_24_218,LEG_24_49,LEG_24_45"
  },
  {
    "id": 233,
    "start_hour": 575,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0233",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_25_80"
  },
  {
    "id": 234,
    "start_hour": 518,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0234",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_23_58,LEG_23_95"
  },
  {
    "id": 235,
    "start_hour": 542,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0235",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_24_234"
  },
  {
    "id": 236,
    "start_hour": 578,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0236",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_25_233,LEG_25_19,LEG_25_196"
  },
  {
    "id": 237,
    "start_hour": 519,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0237",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_23_206,LEG_23_103"
  },
  {
    "id": 238,
    "start_hour": 541,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0238",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_24_191,LEG_24_124"
  },
  {
    "id": 239,
    "start_hour": 539,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0239",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_23_79"
  },
  {
    "id": 240,
    "start_hour": 557,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0240",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_24_56,LEG_24_245"
  },
  {
    "id": 241,
    "start_hour": 530,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0241",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_23_205,LEG_23_204"
  },
  {
    "id": 242,
    "start_hour": 530,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0242",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_23_254,LEG_23_257"
  },
  {
    "id": 243,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0243",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_23_123,LEG_23_120"
  },
  {
    "id": 244,
    "start_hour": 433,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0244",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_19_170"
  },
  {
    "id": 245,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0245",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_19_44,LEG_19_45"
  },
  {
    "id": 246,
    "start_hour": 421,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0246",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_19_40,LEG_19_169"
  },
  {
    "id": 247,
    "start_hour": 431,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0247",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_19_94,LEG_19_182,LEG_19_179"
  },
  {
    "id": 248,
    "start_hour": 456,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0248",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_20_104,LEG_20_227,LEG_20_30"
  },
  {
    "id": 249,
    "start_hour": 443,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0249",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_19_176"
  },
  {
    "id": 250,
    "start_hour": 460,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0250",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_20_174"
  },
  {
    "id": 251,
    "start_hour": 435,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0251",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_19_101,LEG_19_98"
  },
  {
    "id": 252,
    "start_hour": 444,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D0252",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_20_6,LEG_20_158,LEG_20_22"
  },
  {
    "id": 253,
    "start_hour": 474,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0253",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_21_39,LEG_21_224,LEG_21_231"
  },
  {
    "id": 254,
    "start_hour": 440,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0254",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_19_77"
  },
  {
    "id": 255,
    "start_hour": 465,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0255",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_20_47"
  },
  {
    "id": 256,
    "start_hour": 470,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0256",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_21_70"
  },
  {
    "id": 257,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0257",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_19_110,LEG_19_107"
  },
  {
    "id": 258,
    "start_hour": 446,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0258",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_20_147,LEG_20_149,LEG_20_0"
  },
  {
    "id": 259,
    "start_hour": 478,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0259",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_21_229,LEG_21_250,LEG_21_53"
  },
  {
    "id": 260,
    "start_hour": 438,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0260",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_19_217,LEG_19_34,LEG_19_203"
  },
  {
    "id": 261,
    "start_hour": 459,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0261",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_20_124,LEG_20_166"
  },
  {
    "id": 262,
    "start_hour": 468,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D0262",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_21_88,LEG_21_30,LEG_21_21"
  },
  {
    "id": 263,
    "start_hour": 508,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0263",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_22_64"
  },
  {
    "id": 264,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0264",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_19_156,LEG_19_83"
  },
  {
    "id": 265,
    "start_hour": 445,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0265",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_20_105,LEG_20_101"
  },
  {
    "id": 266,
    "start_hour": 468,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0266",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_21_92,LEG_21_226"
  },
  {
    "id": 267,
    "start_hour": 506,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0267",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_22_179,LEG_22_235,LEG_22_228"
  },
  {
    "id": 268,
    "start_hour": 439,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0268",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_19_80,LEG_19_99"
  },
  {
    "id": 269,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0269",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_20_93,LEG_20_106"
  },
  {
    "id": 270,
    "start_hour": 478,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0270",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_21_126,LEG_21_248"
  },
  {
    "id": 271,
    "start_hour": 498,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0271",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_22_55,LEG_22_232,LEG_22_117"
  },
  {
    "id": 272,
    "start_hour": 444,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0272",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_19_60"
  },
  {
    "id": 273,
    "start_hour": 460,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0273",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_20_17"
  },
  {
    "id": 274,
    "start_hour": 468,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D0274",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_21_28,LEG_21_69,LEG_21_37"
  },
  {
    "id": 275,
    "start_hour": 506,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0275",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_22_5,LEG_22_241,LEG_22_45"
  },
  {
    "id": 276,
    "start_hour": 1,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0276",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_01_144,LEG_01_145"
  },
  {
    "id": 277,
    "start_hour": 1,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0277",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_01_62,LEG_01_217"
  },
  {
    "id": 278,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0278",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_01_22,LEG_01_23"
  },
  {
    "id": 279,
    "start_hour": 0,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0279",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_01_219,LEG_01_220"
  },
  {
    "id": 280,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0280",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_01_12,LEG_01_18"
  },
  {
    "id": 281,
    "start_hour": 3,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0281",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_01_83,LEG_01_86"
  },
  {
    "id": 282,
    "start_hour": 0,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0282",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_01_110,LEG_01_115,LEG_01_128"
  },
  {
    "id": 283,
    "start_hour": 24,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0283",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_02_162,LEG_02_131,LEG_02_134"
  },
  {
    "id": 284,
    "start_hour": -1,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0284",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_01_223,LEG_01_46"
  },
  {
    "id": 285,
    "start_hour": 20,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0285",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_02_86,LEG_02_130,LEG_02_137"
  },
  {
    "id": 286,
    "start_hour": 5,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0286",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_01_117,LEG_01_205,LEG_01_151"
  },
  {
    "id": 287,
    "start_hour": 23,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0287",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_02_249,LEG_02_215,LEG_02_143"
  },
  {
    "id": 288,
    "start_hour": 5,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0288",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_01_129,LEG_01_125,LEG_01_87"
  },
  {
    "id": 289,
    "start_hour": 23,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0289",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_02_217,LEG_02_110,LEG_02_107"
  },
  {
    "id": 290,
    "start_hour": 4,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0290",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_01_9"
  },
  {
    "id": 291,
    "start_hour": 20,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0291",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_02_10,LEG_02_71,LEG_02_140"
  },
  {
    "id": 292,
    "start_hour": 4,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0292",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_01_61,LEG_01_103"
  },
  {
    "id": 293,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0293",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_02_124,LEG_02_76"
  },
  {
    "id": 294,
    "start_hour": 3,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0294",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_01_157,LEG_01_24,LEG_01_112"
  },
  {
    "id": 295,
    "start_hour": 13,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0295",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_02_132"
  },
  {
    "id": 296,
    "start_hour": 7,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0296",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_01_210,LEG_01_27"
  },
  {
    "id": 297,
    "start_hour": 14,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0297",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_02_34,LEG_02_30"
  },
  {
    "id": 298,
    "start_hour": 6,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0298",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_01_153,LEG_01_5,LEG_01_59"
  },
  {
    "id": 299,
    "start_hour": 25,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0299",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_02_18,LEG_02_139,LEG_02_237"
  },
  {
    "id": 300,
    "start_hour": 1,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0300",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_01_155,LEG_01_68,LEG_01_34"
  },
  {
    "id": 301,
    "start_hour": 23,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0301",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_02_216,LEG_02_225,LEG_02_144"
  },
  {
    "id": 302,
    "start_hour": 10,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0302",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_01_158"
  },
  {
    "id": 303,
    "start_hour": 7,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0303",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_01_97,LEG_01_99"
  },
  {
    "id": 304,
    "start_hour": 5,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0304",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_01_14,LEG_01_20"
  },
  {
    "id": 305,
    "start_hour": 7,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0305",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_01_57,LEG_01_133"
  },
  {
    "id": 306,
    "start_hour": 4,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0306",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_01_77,LEG_01_76"
  },
  {
    "id": 307,
    "start_hour": 8,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0307",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_01_218,LEG_01_1"
  },
  {
    "id": 308,
    "start_hour": 7,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0308",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_01_124,LEG_01_127"
  },
  {
    "id": 309,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0309",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_01_3,LEG_01_114"
  },
  {
    "id": 310,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0310",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_01_132,LEG_01_88"
  },
  {
    "id": 311,
    "start_hour": 11,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0311",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_01_63"
  },
  {
    "id": 312,
    "start_hour": 29,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0312",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_02_57,LEG_02_247"
  },
  {
    "id": 313,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0313",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_03_211,LEG_03_8"
  },
  {
    "id": 314,
    "start_hour": 5,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0314",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_01_140,LEG_01_141"
  },
  {
    "id": 315,
    "start_hour": 12,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0315",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_02_9,LEG_02_157,LEG_02_98,LEG_02_219,LEG_02_181"
  },
  {
    "id": 316,
    "start_hour": 47,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0316",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_03_249,LEG_03_215,LEG_03_143"
  },
  {
    "id": 317,
    "start_hour": 3,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0317",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_01_148"
  },
  {
    "id": 318,
    "start_hour": 24,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0318",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_02_178,LEG_02_11"
  },
  {
    "id": 319,
    "start_hour": 44,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0319",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_03_10,LEG_03_71,LEG_03_140"
  },
  {
    "id": 320,
    "start_hour": 9,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0320",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_01_53,LEG_01_52"
  },
  {
    "id": 321,
    "start_hour": 28,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0321",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_02_21,LEG_02_7"
  },
  {
    "id": 322,
    "start_hour": 50,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0322",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_03_5"
  },
  {
    "id": 323,
    "start_hour": 2,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0323",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_01_111,LEG_01_113,LEG_01_120"
  },
  {
    "id": 324,
    "start_hour": 27,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0324",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_02_147,LEG_02_133"
  },
  {
    "id": 325,
    "start_hour": 37,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0325",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_03_132"
  },
  {
    "id": 326,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0326",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_01_92"
  },
  {
    "id": 327,
    "start_hour": 26,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0327",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_02_99,LEG_02_242,LEG_02_35"
  },
  {
    "id": 328,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0328",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_01_25"
  },
  {
    "id": 329,
    "start_hour": 35,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0329",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_02_37"
  },
  {
    "id": 330,
    "start_hour": 6,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0330",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_01_84,LEG_01_186,LEG_01_221"
  },
  {
    "id": 331,
    "start_hour": 26,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0331",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_02_158,LEG_02_73,LEG_02_160"
  },
  {
    "id": 332,
    "start_hour": 7,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0332",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_01_209"
  },
  {
    "id": 333,
    "start_hour": 12,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0333",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_02_88,LEG_02_246,LEG_02_227,LEG_02_24"
  },
  {
    "id": 334,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0334",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_01_6"
  },
  {
    "id": 335,
    "start_hour": 26,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0335",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_02_5,LEG_02_145"
  },
  {
    "id": 336,
    "start_hour": 51,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0336",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_03_147,LEG_03_133"
  },
  {
    "id": 337,
    "start_hour": 61,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0337",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_04_132"
  },
  {
    "id": 338,
    "start_hour": 1,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0338",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_01_35,LEG_01_142,LEG_01_191"
  },
  {
    "id": 339,
    "start_hour": 26,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0339",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_02_183,LEG_02_184,LEG_02_6"
  },
  {
    "id": 340,
    "start_hour": 36,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0340",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_03_88,LEG_03_246,LEG_03_65"
  },
  {
    "id": 341,
    "start_hour": 74,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0341",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_04_17"
  },
  {
    "id": 342,
    "start_hour": 4,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0342",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_01_123,LEG_01_175,LEG_01_108"
  },
  {
    "id": 343,
    "start_hour": 26,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0343",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_02_202,LEG_02_201"
  },
  {
    "id": 344,
    "start_hour": 48,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0344",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_03_111,LEG_03_42"
  },
  {
    "id": 345,
    "start_hour": 74,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0345",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_04_33"
  },
  {
    "id": 346,
    "start_hour": 9,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0346",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_01_135,LEG_01_189"
  },
  {
    "id": 347,
    "start_hour": 14,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0347",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_02_235"
  },
  {
    "id": 348,
    "start_hour": 50,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0348",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_03_234,LEG_03_20,LEG_03_200"
  },
  {
    "id": 349,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0349",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_04_211,LEG_04_8"
  },
  {
    "id": 350,
    "start_hour": 0,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0350",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_01_55,LEG_01_118,LEG_01_138"
  },
  {
    "id": 351,
    "start_hour": 23,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0351",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_02_15,LEG_02_177"
  },
  {
    "id": 352,
    "start_hour": 48,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0352",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_03_178,LEG_03_11"
  },
  {
    "id": 353,
    "start_hour": 68,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0353",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_04_10,LEG_04_71,LEG_04_140"
  },
  {
    "id": 354,
    "start_hour": 6,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0354",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_01_204,LEG_01_198"
  },
  {
    "id": 355,
    "start_hour": 13,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D0355",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_02_14,LEG_02_176,LEG_02_240"
  },
  {
    "id": 356,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0356",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_03_243,LEG_03_155"
  },
  {
    "id": 357,
    "start_hour": 72,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0357",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_04_162,LEG_04_131,LEG_04_134"
  },
  {
    "id": 358,
    "start_hour": 9,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0358",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_01_80,LEG_01_150"
  },
  {
    "id": 359,
    "start_hour": 14,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0359",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_02_102,LEG_02_95,LEG_02_255"
  },
  {
    "id": 360,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0360",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_03_158,LEG_03_73,LEG_03_160"
  },
  {
    "id": 361,
    "start_hour": 8,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0361",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_01_152,LEG_01_66"
  },
  {
    "id": 362,
    "start_hour": 27,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0362",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_02_198,LEG_02_199,LEG_02_196,LEG_02_47"
  },
  {
    "id": 363,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0363",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_03_167,LEG_03_4,LEG_03_135"
  },
  {
    "id": 364,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0364",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_01_17"
  },
  {
    "id": 365,
    "start_hour": 28,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0365",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_02_64,LEG_02_192"
  },
  {
    "id": 366,
    "start_hour": 49,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0366",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_03_41,LEG_03_120,LEG_03_115"
  },
  {
    "id": 367,
    "start_hour": 0,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0367",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_01_187,LEG_01_79,LEG_01_121"
  },
  {
    "id": 368,
    "start_hour": 29,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0368",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_02_212,LEG_02_128"
  },
  {
    "id": 369,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0369",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_03_202,LEG_03_201,LEG_03_105"
  },
  {
    "id": 370,
    "start_hour": 3,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0370",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_01_208"
  },
  {
    "id": 371,
    "start_hour": 26,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0371",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_02_243,LEG_02_146"
  },
  {
    "id": 372,
    "start_hour": 53,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0372",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_03_212,LEG_03_128"
  },
  {
    "id": 373,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0373",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_04_202,LEG_04_201,LEG_04_105"
  },
  {
    "id": 374,
    "start_hour": 18,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0374",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_02_40,LEG_02_197"
  },
  {
    "id": 375,
    "start_hour": 52,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0375",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_03_193,LEG_03_192"
  },
  {
    "id": 376,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0376",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_04_41,LEG_04_227,LEG_04_24"
  },
  {
    "id": 377,
    "start_hour": 8,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0377",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_01_15,LEG_01_168"
  },
  {
    "id": 378,
    "start_hour": 15,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0378",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_02_210,LEG_02_203"
  },
  {
    "id": 379,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0379",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_03_122,LEG_03_79"
  },
  {
    "id": 380,
    "start_hour": 77,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0380",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_04_42"
  },
  {
    "id": 381,
    "start_hour": 97,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0381",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_05_67,LEG_05_226"
  },
  {
    "id": 382,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0382",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_05_78,LEG_05_75"
  },
  {
    "id": 383,
    "start_hour": 96,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0383",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_05_228,LEG_05_229"
  },
  {
    "id": 384,
    "start_hour": 87,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0384",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_05_140,LEG_05_168"
  },
  {
    "id": 385,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0385",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_05_195,LEG_05_196"
  },
  {
    "id": 386,
    "start_hour": 99,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0386",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_05_85,LEG_05_88"
  },
  {
    "id": 387,
    "start_hour": 96,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0387",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_05_127,LEG_05_129"
  },
  {
    "id": 388,
    "start_hour": 88,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0388",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_05_105,LEG_05_11"
  },
  {
    "id": 389,
    "start_hour": 87,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0389",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_05_32,LEG_05_198"
  },
  {
    "id": 390,
    "start_hour": 97,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0390",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_05_157,LEG_05_158"
  },
  {
    "id": 391,
    "start_hour": 86,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0391",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_05_31,LEG_05_27"
  },
  {
    "id": 392,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0392",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_05_15,LEG_05_21"
  },
  {
    "id": 393,
    "start_hour": 86,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0393",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_05_22,LEG_05_58"
  },
  {
    "id": 394,
    "start_hour": 109,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0394",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_06_39,LEG_06_151,LEG_06_88,LEG_06_200"
  },
  {
    "id": 395,
    "start_hour": 105,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0395",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_05_149,LEG_05_209"
  },
  {
    "id": 396,
    "start_hour": 109,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0396",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_06_172"
  },
  {
    "id": 397,
    "start_hour": 103,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0397",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_05_135,LEG_05_139"
  },
  {
    "id": 398,
    "start_hour": 100,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0398",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_05_80,LEG_05_79"
  },
  {
    "id": 399,
    "start_hour": 104,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0399",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_05_227,LEG_05_0"
  },
  {
    "id": 400,
    "start_hour": 101,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0400",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_05_17,LEG_05_23"
  },
  {
    "id": 401,
    "start_hour": 88,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0401",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_05_1,LEG_05_54,LEG_05_60"
  },
  {
    "id": 402,
    "start_hour": 124,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0402",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_06_17,LEG_06_118"
  },
  {
    "id": 403,
    "start_hour": 133,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0403",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_07_132"
  },
  {
    "id": 404,
    "start_hour": 101,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0404",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_05_143,LEG_05_137"
  },
  {
    "id": 405,
    "start_hour": 108,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0405",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_06_78,LEG_06_221,LEG_06_99"
  },
  {
    "id": 406,
    "start_hour": 144,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0406",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_07_111,LEG_07_232,LEG_07_138"
  },
  {
    "id": 407,
    "start_hour": 103,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0407",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_05_216,LEG_05_39,LEG_05_206"
  },
  {
    "id": 408,
    "start_hour": 123,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0408",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_06_122,LEG_06_102,LEG_06_104"
  },
  {
    "id": 409,
    "start_hour": 107,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0409",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_05_29"
  },
  {
    "id": 410,
    "start_hour": 131,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0410",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_06_29"
  },
  {
    "id": 411,
    "start_hour": 86,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0411",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_05_211"
  },
  {
    "id": 412,
    "start_hour": 122,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0412",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_06_215,LEG_06_130"
  },
  {
    "id": 413,
    "start_hour": 152,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0413",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_07_8"
  },
  {
    "id": 414,
    "start_hour": 156,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0414",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_08_9,LEG_08_157,LEG_08_98,LEG_08_219"
  },
  {
    "id": 415,
    "start_hour": 107,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0415",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_05_6"
  },
  {
    "id": 416,
    "start_hour": 123,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0416",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_06_3,LEG_06_129"
  },
  {
    "id": 417,
    "start_hour": 147,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0417",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_07_147,LEG_07_133"
  },
  {
    "id": 418,
    "start_hour": 157,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0418",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_08_132"
  },
  {
    "id": 419,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0419",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_05_146,LEG_05_90"
  },
  {
    "id": 420,
    "start_hour": 111,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0420",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_06_121,LEG_06_128,LEG_06_98"
  },
  {
    "id": 421,
    "start_hour": 146,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0421",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_07_99,LEG_07_155"
  },
  {
    "id": 422,
    "start_hour": 168,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0422",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_08_162,LEG_08_131,LEG_08_134"
  },
  {
    "id": 423,
    "start_hour": 107,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0423",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_05_20"
  },
  {
    "id": 424,
    "start_hour": 124,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0424",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_06_56,LEG_06_176"
  },
  {
    "id": 425,
    "start_hour": 145,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0425",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_07_41,LEG_07_42"
  },
  {
    "id": 426,
    "start_hour": 170,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0426",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_08_33"
  },
  {
    "id": 427,
    "start_hour": 95,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0427",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_05_232,LEG_05_52"
  },
  {
    "id": 428,
    "start_hour": 116,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0428",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_06_75,LEG_06_160"
  },
  {
    "id": 429,
    "start_hour": 144,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0429",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_07_178,LEG_07_11"
  },
  {
    "id": 430,
    "start_hour": 164,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0430",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_08_10,LEG_08_71,LEG_08_140"
  },
  {
    "id": 431,
    "start_hour": 105,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0431",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_05_82,LEG_05_162"
  },
  {
    "id": 432,
    "start_hour": 110,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0432",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_06_146,LEG_06_148,LEG_06_237"
  },
  {
    "id": 433,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0433",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_07_158,LEG_07_73,LEG_07_160"
  },
  {
    "id": 434,
    "start_hour": 101,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0434",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_05_154,LEG_05_155,LEG_05_69"
  },
  {
    "id": 435,
    "start_hour": 123,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0435",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_06_182,LEG_06_183,LEG_06_180,LEG_06_38"
  },
  {
    "id": 436,
    "start_hour": 145,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0436",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_07_167,LEG_07_4,LEG_07_135"
  },
  {
    "id": 437,
    "start_hour": 84,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0437",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_05_65,LEG_05_114,LEG_05_115"
  },
  {
    "id": 438,
    "start_hour": 122,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0438",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_06_186,LEG_06_189"
  },
  {
    "id": 439,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0439",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_07_72,LEG_07_243,LEG_07_35"
  },
  {
    "id": 440,
    "start_hour": 386,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0440",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_17_254,LEG_17_257"
  },
  {
    "id": 441,
    "start_hour": 386,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0441",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_17_205,LEG_17_204"
  },
  {
    "id": 442,
    "start_hour": 375,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0442",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_17_206,LEG_17_103"
  },
  {
    "id": 443,
    "start_hour": 397,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0443",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_18_191,LEG_18_124"
  },
  {
    "id": 444,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0444",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_17_123,LEG_17_120"
  },
  {
    "id": 445,
    "start_hour": 374,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0445",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_17_58,LEG_17_95"
  },
  {
    "id": 446,
    "start_hour": 398,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0446",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_18_234"
  },
  {
    "id": 447,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0447",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_19_207,LEG_19_18"
  },
  {
    "id": 448,
    "start_hour": 455,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0448",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_20_90,LEG_20_133,LEG_20_188"
  },
  {
    "id": 449,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0449",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_17_202,LEG_17_148"
  },
  {
    "id": 450,
    "start_hour": 398,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0450",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_18_176,LEG_18_178"
  },
  {
    "id": 451,
    "start_hour": 420,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0451",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_19_136,LEG_19_95,LEG_19_70"
  },
  {
    "id": 452,
    "start_hour": 458,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0452",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_20_113"
  },
  {
    "id": 453,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0453",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_17_207,LEG_17_8"
  },
  {
    "id": 454,
    "start_hour": 409,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0454",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_18_44,LEG_18_168,LEG_18_54"
  },
  {
    "id": 455,
    "start_hour": 438,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0455",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_19_3,LEG_19_7"
  },
  {
    "id": 456,
    "start_hour": 459,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0456",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_20_3,LEG_20_157,LEG_20_184"
  },
  {
    "id": 457,
    "start_hour": 394,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0457",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_17_256"
  },
  {
    "id": 458,
    "start_hour": 397,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D0458",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_18_217,LEG_18_218,LEG_18_49"
  },
  {
    "id": 459,
    "start_hour": 420,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0459",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_19_74,LEG_19_209,LEG_19_53,LEG_19_76,LEG_19_198"
  },
  {
    "id": 460,
    "start_hour": 458,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0460",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_20_165,LEG_20_16,LEG_20_181"
  },
  {
    "id": 461,
    "start_hour": 395,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0461",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_17_128"
  },
  {
    "id": 462,
    "start_hour": 414,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0462",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_18_126,LEG_18_128"
  },
  {
    "id": 463,
    "start_hour": 443,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0463",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_19_68"
  },
  {
    "id": 464,
    "start_hour": 461,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0464",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_20_46,LEG_20_228"
  },
  {
    "id": 465,
    "start_hour": 302,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0465",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_14_35,LEG_14_31"
  },
  {
    "id": 466,
    "start_hour": 303,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0466",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_14_151,LEG_14_184"
  },
  {
    "id": 467,
    "start_hour": 313,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0467",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_14_170,LEG_14_171"
  },
  {
    "id": 468,
    "start_hour": 312,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0468",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_14_254,LEG_14_255"
  },
  {
    "id": 469,
    "start_hour": 303,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0469",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_14_36,LEG_14_220"
  },
  {
    "id": 470,
    "start_hour": 312,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0470",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_14_138,LEG_14_140"
  },
  {
    "id": 471,
    "start_hour": 304,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0471",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_14_115,LEG_14_13"
  },
  {
    "id": 472,
    "start_hour": 315,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0472",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_14_95,LEG_14_98"
  },
  {
    "id": 473,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0473",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_14_17,LEG_14_24"
  },
  {
    "id": 474,
    "start_hour": 311,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0474",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_14_258,LEG_14_59"
  },
  {
    "id": 475,
    "start_hour": 332,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0475",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_15_85,LEG_15_129,LEG_15_134"
  },
  {
    "id": 476,
    "start_hour": 316,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0476",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_14_91,LEG_14_57"
  },
  {
    "id": 477,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0477",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_15_207,LEG_15_8"
  },
  {
    "id": 478,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0478",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_14_78,LEG_14_252,LEG_14_178"
  },
  {
    "id": 479,
    "start_hour": 335,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0479",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_15_247,LEG_15_211,LEG_15_139"
  },
  {
    "id": 480,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0480",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_14_156,LEG_14_101"
  },
  {
    "id": 481,
    "start_hour": 320,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0481",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_14_253,LEG_14_2"
  },
  {
    "id": 482,
    "start_hour": 319,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0482",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_14_146,LEG_14_150"
  },
  {
    "id": 483,
    "start_hour": 317,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0483",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_14_20,LEG_14_27"
  },
  {
    "id": 484,
    "start_hour": 319,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0484",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_14_113,LEG_14_116"
  },
  {
    "id": 485,
    "start_hour": 300,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0485",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_14_74,LEG_14_126,LEG_14_127"
  },
  {
    "id": 486,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0486",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_15_198,LEG_15_195,LEG_15_192,LEG_15_46"
  },
  {
    "id": 487,
    "start_hour": 359,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0487",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_16_186,LEG_16_226,LEG_16_140"
  },
  {
    "id": 488,
    "start_hour": 323,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0488",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_14_107"
  },
  {
    "id": 489,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0489",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_15_98,LEG_15_97,LEG_15_213,LEG_15_177"
  },
  {
    "id": 490,
    "start_hour": 359,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0490",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_16_247,LEG_16_211,LEG_16_139"
  },
  {
    "id": 491,
    "start_hour": 304,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0491",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_14_3,LEG_14_61,LEG_14_38"
  },
  {
    "id": 492,
    "start_hour": 338,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0492",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_15_5,LEG_15_151"
  },
  {
    "id": 493,
    "start_hour": 360,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0493",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_16_158,LEG_16_130,LEG_16_133"
  },
  {
    "id": 494,
    "start_hour": 323,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0494",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_14_33"
  },
  {
    "id": 495,
    "start_hour": 344,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0495",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_15_29,LEG_15_75"
  },
  {
    "id": 496,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0496",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_16_17,LEG_16_136,LEG_16_236,LEG_16_177"
  },
  {
    "id": 497,
    "start_hour": 383,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0497",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_17_247,LEG_17_211,LEG_17_139"
  },
  {
    "id": 498,
    "start_hour": 321,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0498",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_14_92,LEG_14_176"
  },
  {
    "id": 499,
    "start_hour": 326,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0499",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_15_157,LEG_15_160,LEG_15_141"
  },
  {
    "id": 500,
    "start_hour": 363,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0500",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_16_143,LEG_16_132"
  },
  {
    "id": 501,
    "start_hour": 373,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0501",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_17_131"
  },
  {
    "id": 502,
    "start_hour": 317,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0502",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_14_153,LEG_14_148,LEG_14_231"
  },
  {
    "id": 503,
    "start_hour": 336,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0503",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_15_238,LEG_15_173"
  },
  {
    "id": 504,
    "start_hour": 360,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0504",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_16_174,LEG_16_11"
  },
  {
    "id": 505,
    "start_hour": 380,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0505",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_17_10,LEG_17_71,LEG_17_137"
  },
  {
    "id": 506,
    "start_hour": 323,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0506",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_14_8"
  },
  {
    "id": 507,
    "start_hour": 340,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0507",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_15_42,LEG_15_188"
  },
  {
    "id": 508,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0508",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_16_40,LEG_16_49,LEG_16_30,LEG_16_0"
  },
  {
    "id": 509,
    "start_hour": 382,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0509",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_17_227,LEG_17_226,LEG_17_140"
  },
  {
    "id": 510,
    "start_hour": 321,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0510",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_14_160,LEG_14_219"
  },
  {
    "id": 511,
    "start_hour": 326,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0511",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_15_24,LEG_15_66"
  },
  {
    "id": 512,
    "start_hour": 349,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0512",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_16_217,LEG_16_218,LEG_16_167,LEG_16_23"
  },
  {
    "id": 513,
    "start_hour": 302,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0513",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_14_26,LEG_14_65"
  },
  {
    "id": 514,
    "start_hour": 325,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0514",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_15_13,LEG_15_172,LEG_15_239"
  },
  {
    "id": 515,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0515",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_16_242,LEG_16_93,LEG_16_175"
  },
  {
    "id": 516,
    "start_hour": 317,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0516",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_14_165,LEG_14_166"
  },
  {
    "id": 517,
    "start_hour": 324,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D0517",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_15_28,LEG_15_69,LEG_15_68"
  },
  {
    "id": 518,
    "start_hour": 364,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0518",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_16_20,LEG_16_253"
  },
  {
    "id": 519,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0519",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_17_154,LEG_17_4,LEG_17_215"
  },
  {
    "id": 520,
    "start_hour": 322,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0520",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_14_172,LEG_14_200"
  },
  {
    "id": 521,
    "start_hour": 326,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0521",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_15_121,LEG_15_76"
  },
  {
    "id": 522,
    "start_hour": 348,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0522",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_16_87,LEG_16_34"
  },
  {
    "id": 523,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0523",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_17_105,LEG_17_161,LEG_17_73,LEG_17_156"
  },
  {
    "id": 524,
    "start_hour": 302,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0524",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_14_234"
  },
  {
    "id": 525,
    "start_hour": 338,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0525",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_15_233,LEG_15_142"
  },
  {
    "id": 526,
    "start_hour": 365,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0526",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_16_208,LEG_16_127"
  },
  {
    "id": 527,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0527",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_17_198,LEG_17_197,LEG_17_104"
  },
  {
    "id": 528,
    "start_hour": 461,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0528",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_20_142,LEG_20_136,LEG_20_95"
  },
  {
    "id": 529,
    "start_hour": 480,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0529",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_21_26,LEG_21_241"
  },
  {
    "id": 530,
    "start_hour": 506,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0530",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_22_242,LEG_22_253"
  },
  {
    "id": 531,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0531",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_23_154,LEG_23_73,LEG_23_156"
  },
  {
    "id": 532,
    "start_hour": 446,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0532",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_20_226,LEG_20_223"
  },
  {
    "id": 533,
    "start_hour": 457,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0533",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_20_155,LEG_20_156"
  },
  {
    "id": 534,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0534",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_20_13,LEG_20_19"
  },
  {
    "id": 535,
    "start_hour": 447,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0535",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_20_139,LEG_20_168"
  },
  {
    "id": 536,
    "start_hour": 456,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0536",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_20_233,LEG_20_234"
  },
  {
    "id": 537,
    "start_hour": 444,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0537",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_20_63,LEG_20_60"
  },
  {
    "id": 538,
    "start_hour": 459,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0538",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_20_91,LEG_20_94"
  },
  {
    "id": 539,
    "start_hour": 456,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0539",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_20_127,LEG_20_128"
  },
  {
    "id": 540,
    "start_hour": 460,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0540",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_20_66,LEG_20_111"
  },
  {
    "id": 541,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0541",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_21_125,LEG_21_77"
  },
  {
    "id": 542,
    "start_hour": 455,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0542",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_20_237,LEG_20_49"
  },
  {
    "id": 543,
    "start_hour": 476,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0543",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_21_86,LEG_21_131,LEG_21_136"
  },
  {
    "id": 544,
    "start_hour": 460,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0544",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_20_86,LEG_20_85"
  },
  {
    "id": 545,
    "start_hour": 461,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0545",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_20_15,LEG_20_21"
  },
  {
    "id": 546,
    "start_hour": 464,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0546",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_20_232,LEG_20_1"
  },
  {
    "id": 547,
    "start_hour": 463,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0547",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_20_134,LEG_20_138"
  },
  {
    "id": 548,
    "start_hour": 444,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D0548",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_20_23,LEG_20_218,LEG_20_58"
  },
  {
    "id": 549,
    "start_hour": 484,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0549",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_21_20,LEG_21_134"
  },
  {
    "id": 550,
    "start_hour": 493,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0550",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_22_131"
  },
  {
    "id": 551,
    "start_hour": 446,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0551",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_20_20,LEG_20_55"
  },
  {
    "id": 552,
    "start_hour": 469,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0552",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_21_48,LEG_21_50,LEG_21_65"
  },
  {
    "id": 553,
    "start_hour": 506,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0553",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_22_16"
  },
  {
    "id": 554,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0554",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_20_169,LEG_20_79"
  },
  {
    "id": 555,
    "start_hour": 489,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0555",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_21_57"
  },
  {
    "id": 556,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0556",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_22_207,LEG_22_8"
  },
  {
    "id": 557,
    "start_hour": 461,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0557",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_20_126,LEG_20_125,LEG_20_64"
  },
  {
    "id": 558,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0558",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_21_17,LEG_21_138,LEG_21_238,LEG_21_179"
  },
  {
    "id": 559,
    "start_hour": 503,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0559",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_22_247,LEG_22_211,LEG_22_139"
  },
  {
    "id": 560,
    "start_hour": 467,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0560",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_20_26"
  },
  {
    "id": 561,
    "start_hour": 491,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0561",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_21_36"
  },
  {
    "id": 562,
    "start_hour": 461,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0562",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_20_152,LEG_20_153"
  },
  {
    "id": 563,
    "start_hour": 468,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0563",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_21_9,LEG_21_155,LEG_21_153"
  },
  {
    "id": 564,
    "start_hour": 504,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0564",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_22_158,LEG_22_11"
  },
  {
    "id": 565,
    "start_hour": 524,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0565",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_23_10,LEG_23_71,LEG_23_137"
  },
  {
    "id": 566,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0566",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_20_119,LEG_20_129,LEG_20_167"
  },
  {
    "id": 567,
    "start_hour": 479,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0567",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_21_110,LEG_21_111"
  },
  {
    "id": 568,
    "start_hour": 504,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0568",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_22_110,LEG_22_41"
  },
  {
    "id": 569,
    "start_hour": 530,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0569",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_23_32"
  },
  {
    "id": 570,
    "start_hour": 465,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0570",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_20_88,LEG_20_161"
  },
  {
    "id": 571,
    "start_hour": 470,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0571",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_21_24,LEG_21_66"
  },
  {
    "id": 572,
    "start_hour": 493,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0572",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_22_48,LEG_22_50,LEG_22_65"
  },
  {
    "id": 573,
    "start_hour": 530,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0573",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_23_16"
  },
  {
    "id": 574,
    "start_hour": 467,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0574",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_20_102"
  },
  {
    "id": 575,
    "start_hour": 482,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0575",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_21_100,LEG_21_144"
  },
  {
    "id": 576,
    "start_hour": 509,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0576",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_22_208,LEG_22_127"
  },
  {
    "id": 577,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0577",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_23_198,LEG_23_197,LEG_23_104"
  },
  {
    "id": 578,
    "start_hour": 463,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0578",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_20_87,LEG_20_69"
  },
  {
    "id": 579,
    "start_hour": 470,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0579",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_21_178,LEG_21_180,LEG_21_82"
  },
  {
    "id": 580,
    "start_hour": 507,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0580",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_22_194,LEG_22_195,LEG_22_192,LEG_22_46"
  },
  {
    "id": 581,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0581",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_23_163,LEG_23_4,LEG_23_215"
  },
  {
    "id": 582,
    "start_hour": 467,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0582",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_20_5"
  },
  {
    "id": 583,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0583",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_21_5,LEG_21_182,LEG_21_6"
  },
  {
    "id": 584,
    "start_hour": 492,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0584",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_22_87,LEG_22_34"
  },
  {
    "id": 585,
    "start_hour": 518,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0585",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_23_33,LEG_23_36"
  },
  {
    "id": 586,
    "start_hour": 465,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0586",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_20_148,LEG_20_199"
  },
  {
    "id": 587,
    "start_hour": 470,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0587",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_21_236"
  },
  {
    "id": 588,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0588",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_22_233,LEG_22_141"
  },
  {
    "id": 589,
    "start_hour": 531,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0589",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_23_143,LEG_23_93,LEG_23_175"
  },
  {
    "id": 590,
    "start_hour": 395,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0590",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_17_7"
  },
  {
    "id": 591,
    "start_hour": 410,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0591",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_18_5,LEG_18_142"
  },
  {
    "id": 592,
    "start_hour": 437,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0592",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_19_190,LEG_19_118"
  },
  {
    "id": 593,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0593",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_20_183,LEG_20_182,LEG_20_99"
  },
  {
    "id": 594,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0594",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_17_44,LEG_17_168,LEG_17_54"
  },
  {
    "id": 595,
    "start_hour": 414,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0595",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_18_3,LEG_18_253"
  },
  {
    "id": 596,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0596",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_19_145,LEG_19_64,LEG_19_147"
  },
  {
    "id": 597,
    "start_hour": 374,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0597",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_17_243,LEG_17_240"
  },
  {
    "id": 598,
    "start_hour": 376,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0598",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_17_115,LEG_17_12"
  },
  {
    "id": 599,
    "start_hour": 387,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0599",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_17_96,LEG_17_99"
  },
  {
    "id": 600,
    "start_hour": 384,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0600",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_17_251,LEG_17_252"
  },
  {
    "id": 601,
    "start_hour": 374,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0601",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_17_33,LEG_17_29"
  },
  {
    "id": 602,
    "start_hour": 375,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0602",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_17_150,LEG_17_183"
  },
  {
    "id": 603,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0603",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_17_15,LEG_17_22"
  },
  {
    "id": 604,
    "start_hour": 375,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0604",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_17_35,LEG_17_216"
  },
  {
    "id": 605,
    "start_hour": 385,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0605",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_17_78,LEG_17_249"
  },
  {
    "id": 606,
    "start_hour": 383,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0606",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_17_255,LEG_17_59"
  },
  {
    "id": 607,
    "start_hour": 404,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0607",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_18_85,LEG_18_129,LEG_18_134"
  },
  {
    "id": 608,
    "start_hour": 388,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0608",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_17_77"
  },
  {
    "id": 609,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0609",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_18_123,LEG_18_76"
  },
  {
    "id": 610,
    "start_hour": 391,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0610",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_17_113,LEG_17_116"
  },
  {
    "id": 611,
    "start_hour": 391,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0611",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_17_145,LEG_17_149"
  },
  {
    "id": 612,
    "start_hour": 389,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0612",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_17_18,LEG_17_25"
  },
  {
    "id": 613,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0613",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_17_155,LEG_17_102"
  },
  {
    "id": 614,
    "start_hour": 388,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0614",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_17_92,LEG_17_91"
  },
  {
    "id": 615,
    "start_hour": 392,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0615",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_17_250,LEG_17_1"
  },
  {
    "id": 616,
    "start_hour": 393,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0616",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_17_93,LEG_17_175"
  },
  {
    "id": 617,
    "start_hour": 398,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0617",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_18_24,LEG_18_66"
  },
  {
    "id": 618,
    "start_hour": 421,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0618",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_19_13,LEG_19_168,LEG_19_143,LEG_19_137"
  },
  {
    "id": 619,
    "start_hour": 395,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0619",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_17_31"
  },
  {
    "id": 620,
    "start_hour": 419,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0620",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_18_36"
  },
  {
    "id": 621,
    "start_hour": 372,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0621",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_17_74,LEG_17_126,LEG_17_237"
  },
  {
    "id": 622,
    "start_hour": 397,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0622",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_18_47,LEG_18_194,LEG_18_202,LEG_18_148"
  },
  {
    "id": 623,
    "start_hour": 385,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0623",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_17_169,LEG_17_220,LEG_17_63"
  },
  {
    "id": 624,
    "start_hour": 406,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0624",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_18_225,LEG_18_109"
  },
  {
    "id": 625,
    "start_hour": 432,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0625",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_19_102,LEG_19_49"
  },
  {
    "id": 626,
    "start_hour": 457,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0626",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_20_57,LEG_20_39,LEG_20_25"
  },
  {
    "id": 627,
    "start_hour": 393,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0627",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_17_159,LEG_17_214"
  },
  {
    "id": 628,
    "start_hour": 398,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0628",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_18_243,LEG_18_240"
  },
  {
    "id": 629,
    "start_hour": 420,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0629",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_19_155"
  },
  {
    "id": 630,
    "start_hour": 455,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0630",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_20_12,LEG_20_198,LEG_20_197"
  },
  {
    "id": 631,
    "start_hour": 389,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0631",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_17_164,LEG_17_165"
  },
  {
    "id": 632,
    "start_hour": 396,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0632",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_18_166,LEG_18_185"
  },
  {
    "id": 633,
    "start_hour": 422,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0633",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_19_91,LEG_19_85,LEG_19_123"
  },
  {
    "id": 634,
    "start_hour": 445,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0634",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_20_120"
  },
  {
    "id": 635,
    "start_hour": 395,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0635",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_17_107"
  },
  {
    "id": 636,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0636",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_18_98,LEG_18_151,LEG_18_46"
  },
  {
    "id": 637,
    "start_hour": 433,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0637",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_19_152,LEG_19_38,LEG_19_105,LEG_19_104"
  },
  {
    "id": 638,
    "start_hour": 455,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0638",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_20_98,LEG_20_78"
  },
  {
    "id": 639,
    "start_hour": 386,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0639",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_17_130,LEG_17_133,LEG_17_141"
  },
  {
    "id": 640,
    "start_hour": 411,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0640",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_18_143,LEG_18_141"
  },
  {
    "id": 641,
    "start_hour": 435,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0641",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_19_133,LEG_19_84,LEG_19_162"
  },
  {
    "id": 642,
    "start_hour": 389,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0642",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_17_152,LEG_17_147"
  },
  {
    "id": 643,
    "start_hour": 396,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D0643",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_18_28,LEG_18_69,LEG_18_68"
  },
  {
    "id": 644,
    "start_hour": 436,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0644",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_19_19,LEG_19_226"
  },
  {
    "id": 645,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0645",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_20_144,LEG_20_62,LEG_20_146"
  },
  {
    "id": 646,
    "start_hour": 384,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0646",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_17_138,LEG_17_210,LEG_17_43"
  },
  {
    "id": 647,
    "start_hour": 407,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0647",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_18_212,LEG_18_187"
  },
  {
    "id": 648,
    "start_hour": 440,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0648",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_19_173"
  },
  {
    "id": 649,
    "start_hour": 445,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0649",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_20_11,LEG_20_171,LEG_20_145,LEG_20_97"
  },
  {
    "id": 650,
    "start_hour": 221,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0650",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_10_152,LEG_10_147"
  },
  {
    "id": 651,
    "start_hour": 228,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D0651",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_11_29,LEG_11_68"
  },
  {
    "id": 652,
    "start_hour": 268,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0652",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_12_21,LEG_12_225"
  },
  {
    "id": 653,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0653",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_13_145,LEG_13_67,LEG_13_147"
  },
  {
    "id": 654,
    "start_hour": 207,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0654",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_10_36,LEG_10_217"
  },
  {
    "id": 655,
    "start_hour": 208,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0655",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_10_114,LEG_10_12"
  },
  {
    "id": 656,
    "start_hour": 207,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0656",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_10_150,LEG_10_183"
  },
  {
    "id": 657,
    "start_hour": 217,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0657",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_10_169,LEG_10_170"
  },
  {
    "id": 658,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0658",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_10_16,LEG_10_23"
  },
  {
    "id": 659,
    "start_hour": 219,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0659",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_10_95,LEG_10_98"
  },
  {
    "id": 660,
    "start_hour": 216,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0660",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_10_249,LEG_10_250"
  },
  {
    "id": 661,
    "start_hour": 217,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0661",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_10_76,LEG_10_247"
  },
  {
    "id": 662,
    "start_hour": 220,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0662",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_10_75,LEG_10_119"
  },
  {
    "id": 663,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0663",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_11_209,LEG_11_8"
  },
  {
    "id": 664,
    "start_hour": 206,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0664",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_10_34,LEG_10_30,LEG_10_73"
  },
  {
    "id": 665,
    "start_hour": 241,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0665",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_11_18,LEG_11_139,LEG_11_235"
  },
  {
    "id": 666,
    "start_hour": 215,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0666",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_10_253,LEG_10_60"
  },
  {
    "id": 667,
    "start_hour": 236,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0667",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_11_86,LEG_11_71,LEG_11_140"
  },
  {
    "id": 668,
    "start_hour": 227,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0668",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_10_32"
  },
  {
    "id": 669,
    "start_hour": 248,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0669",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_11_30"
  },
  {
    "id": 670,
    "start_hour": 220,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0670",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_10_91,LEG_10_90"
  },
  {
    "id": 671,
    "start_hour": 224,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0671",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_10_248,LEG_10_1"
  },
  {
    "id": 672,
    "start_hour": 223,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0672",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_10_145,LEG_10_149"
  },
  {
    "id": 673,
    "start_hour": 223,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0673",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_10_112,LEG_10_115"
  },
  {
    "id": 674,
    "start_hour": 221,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0674",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_10_19,LEG_10_26"
  },
  {
    "id": 675,
    "start_hour": 225,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0675",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_10_159,LEG_10_216"
  },
  {
    "id": 676,
    "start_hour": 230,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0676",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_11_233"
  },
  {
    "id": 677,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0677",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_12_207,LEG_12_89,LEG_12_195"
  },
  {
    "id": 678,
    "start_hour": 206,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0678",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_10_231"
  },
  {
    "id": 679,
    "start_hour": 242,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0679",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_11_232,LEG_11_98,LEG_11_217"
  },
  {
    "id": 680,
    "start_hour": 263,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0680",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_12_94,LEG_12_13"
  },
  {
    "id": 681,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0681",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_10_155,LEG_10_101"
  },
  {
    "id": 682,
    "start_hour": 231,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0682",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_11_136,LEG_11_226,LEG_11_65"
  },
  {
    "id": 683,
    "start_hour": 266,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0683",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_12_18"
  },
  {
    "id": 684,
    "start_hour": 206,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0684",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_10_240,LEG_10_237"
  },
  {
    "id": 685,
    "start_hour": 228,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0685",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_11_168,LEG_11_187"
  },
  {
    "id": 686,
    "start_hour": 254,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0686",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_12_92,LEG_12_86,LEG_12_122"
  },
  {
    "id": 687,
    "start_hour": 277,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0687",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_13_119"
  },
  {
    "id": 688,
    "start_hour": 221,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0688",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_10_164,LEG_10_165,LEG_10_80"
  },
  {
    "id": 689,
    "start_hour": 243,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0689",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_11_196,LEG_11_197,LEG_11_194,LEG_11_47"
  },
  {
    "id": 690,
    "start_hour": 263,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0690",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_12_170,LEG_12_60"
  },
  {
    "id": 691,
    "start_hour": 290,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0691",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_13_15"
  },
  {
    "id": 692,
    "start_hour": 227,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0692",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_10_7"
  },
  {
    "id": 693,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0693",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_11_5,LEG_11_143"
  },
  {
    "id": 694,
    "start_hour": 267,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0694",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_12_133,LEG_12_149,LEG_12_206"
  },
  {
    "id": 695,
    "start_hour": 277,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0695",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_13_12,LEG_13_178,LEG_13_143,LEG_13_137"
  },
  {
    "id": 696,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0696",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_10_45,LEG_10_167,LEG_10_219"
  },
  {
    "id": 697,
    "start_hour": 242,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0697",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_11_181,LEG_11_164"
  },
  {
    "id": 698,
    "start_hour": 263,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0698",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_12_16,LEG_12_160"
  },
  {
    "id": 699,
    "start_hour": 288,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0699",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_13_163,LEG_13_175,LEG_13_42,LEG_13_27"
  },
  {
    "id": 700,
    "start_hour": 217,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0700",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_10_182,LEG_10_82,LEG_10_188"
  },
  {
    "id": 701,
    "start_hour": 241,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0701",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_11_41,LEG_11_87"
  },
  {
    "id": 702,
    "start_hour": 272,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0702",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_12_80,LEG_12_72"
  },
  {
    "id": 703,
    "start_hour": 289,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0703",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_13_153,LEG_13_208,LEG_13_122"
  },
  {
    "id": 704,
    "start_hour": 225,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0704",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_10_92,LEG_10_175"
  },
  {
    "id": 705,
    "start_hour": 230,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D0705",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_11_159,LEG_11_162"
  },
  {
    "id": 706,
    "start_hour": 252,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0706",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_12_156,LEG_12_168"
  },
  {
    "id": 707,
    "start_hour": 279,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0707",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_13_123,LEG_13_121,LEG_13_88,LEG_13_164"
  },
  {
    "id": 708,
    "start_hour": 562,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0708",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_24_256"
  },
  {
    "id": 709,
    "start_hour": 565,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0709",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_25_217,LEG_25_218,LEG_25_90"
  },
  {
    "id": 710,
    "start_hour": 609,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0710",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_26_51"
  },
  {
    "id": 711,
    "start_hour": 615,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0711",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_27_169,LEG_27_38,LEG_27_209"
  },
  {
    "id": 712,
    "start_hour": 563,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0712",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_24_128"
  },
  {
    "id": 713,
    "start_hour": 587,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0713",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_25_79"
  },
  {
    "id": 714,
    "start_hour": 605,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0714",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_26_50,LEG_26_225"
  },
  {
    "id": 715,
    "start_hour": 543,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D0715",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_24_206,LEG_24_103,LEG_24_37"
  },
  {
    "id": 716,
    "start_hour": 578,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0716",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_25_5,LEG_25_142"
  },
  {
    "id": 717,
    "start_hour": 605,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0717",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_26_190"
  },
  {
    "id": 718,
    "start_hour": 542,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0718",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_24_58,LEG_24_95"
  },
  {
    "id": 719,
    "start_hour": 566,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0719",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_25_33,LEG_25_29,LEG_25_81"
  },
  {
    "id": 720,
    "start_hour": 603,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0720",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_26_177"
  },
  {
    "id": 721,
    "start_hour": 554,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0721",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_24_205,LEG_24_204"
  },
  {
    "id": 722,
    "start_hour": 554,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0722",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_24_254,LEG_24_257"
  },
  {
    "id": 723,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0723",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_24_123,LEG_24_120"
  },
  {
    "id": 724,
    "start_hour": 335,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0724",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_15_51,LEG_15_52"
  },
  {
    "id": 725,
    "start_hour": 325,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0725",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_15_47,LEG_15_186"
  },
  {
    "id": 726,
    "start_hour": 337,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0726",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_15_187"
  },
  {
    "id": 727,
    "start_hour": 347,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0727",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_15_193"
  },
  {
    "id": 728,
    "start_hour": 364,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0728",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_16_189"
  },
  {
    "id": 729,
    "start_hour": 344,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0729",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_15_223"
  },
  {
    "id": 730,
    "start_hour": 362,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0730",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_16_179,LEG_16_235,LEG_16_228"
  },
  {
    "id": 731,
    "start_hour": 337,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0731",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_15_67"
  },
  {
    "id": 732,
    "start_hour": 354,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0732",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_16_62"
  },
  {
    "id": 733,
    "start_hour": 339,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0733",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_15_60,LEG_15_89,LEG_15_209,LEG_15_84"
  },
  {
    "id": 734,
    "start_hour": 335,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0734",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_15_80"
  },
  {
    "id": 735,
    "start_hour": 357,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0735",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_16_125,LEG_16_246"
  },
  {
    "id": 736,
    "start_hour": 378,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0736",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_17_55,LEG_17_248,LEG_17_53"
  },
  {
    "id": 737,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0737",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_15_167,LEG_15_23"
  },
  {
    "id": 738,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0738",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_16_105,LEG_16_161,LEG_16_180,LEG_16_21"
  },
  {
    "id": 739,
    "start_hour": 388,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0739",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_17_64"
  },
  {
    "id": 740,
    "start_hour": 344,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0740",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_15_65"
  },
  {
    "id": 741,
    "start_hour": 362,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0741",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_16_16,LEG_16_162"
  },
  {
    "id": 742,
    "start_hour": 383,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0742",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_17_14,LEG_17_184"
  },
  {
    "id": 743,
    "start_hour": 340,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0743",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_15_230,LEG_15_135"
  },
  {
    "id": 744,
    "start_hour": 348,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0744",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_16_9,LEG_16_153,LEG_16_77"
  },
  {
    "id": 745,
    "start_hour": 380,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0745",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_17_70,LEG_17_182,LEG_17_83"
  },
  {
    "id": 746,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0746",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_15_109,LEG_15_106,LEG_15_231"
  },
  {
    "id": 747,
    "start_hour": 360,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0747",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_16_238,LEG_16_27"
  },
  {
    "id": 748,
    "start_hour": 378,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0748",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_17_39,LEG_17_222,LEG_17_229"
  },
  {
    "id": 749,
    "start_hour": 345,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0749",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_15_219,LEG_15_111"
  },
  {
    "id": 750,
    "start_hour": 349,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0750",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_16_112,LEG_16_106,LEG_16_181"
  },
  {
    "id": 751,
    "start_hour": 383,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0751",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_17_108,LEG_17_118,LEG_17_224"
  },
  {
    "id": 752,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0752",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_15_119,LEG_15_114"
  },
  {
    "id": 753,
    "start_hour": 350,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0753",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_16_157,LEG_16_160,LEG_16_141"
  },
  {
    "id": 754,
    "start_hour": 387,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0754",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_17_143,LEG_17_181"
  },
  {
    "id": 755,
    "start_hour": 407,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0755",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_18_108,LEG_18_118,LEG_18_224"
  },
  {
    "id": 756,
    "start_hour": 344,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0756",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_15_82"
  },
  {
    "id": 757,
    "start_hour": 362,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0757",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_16_72,LEG_16_151"
  },
  {
    "id": 758,
    "start_hour": 384,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0758",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_17_158,LEG_17_27"
  },
  {
    "id": 759,
    "start_hour": 402,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0759",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_18_39,LEG_18_222,LEG_18_229"
  },
  {
    "id": 760,
    "start_hour": 341,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0760",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_15_86"
  },
  {
    "id": 761,
    "start_hour": 369,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0761",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_16_57"
  },
  {
    "id": 762,
    "start_hour": 374,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0762",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_17_121,LEG_17_76"
  },
  {
    "id": 763,
    "start_hour": 396,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0763",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_18_9,LEG_18_153,LEG_18_241,LEG_18_45"
  },
  {
    "id": 764,
    "start_hour": 347,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0764",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_15_37"
  },
  {
    "id": 765,
    "start_hour": 364,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0765",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_16_42,LEG_16_63"
  },
  {
    "id": 766,
    "start_hour": 382,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0766",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_17_225,LEG_17_67"
  },
  {
    "id": 767,
    "start_hour": 402,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0767",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_18_62"
  },
  {
    "id": 768,
    "start_hour": 53,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0768",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_03_156,LEG_03_151,LEG_03_101"
  },
  {
    "id": 769,
    "start_hour": 71,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0769",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_04_217,LEG_04_191"
  },
  {
    "id": 770,
    "start_hour": 104,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0770",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_05_175"
  },
  {
    "id": 771,
    "start_hour": 109,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0771",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_06_11,LEG_06_174,LEG_06_144,LEG_06_93"
  },
  {
    "id": 772,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0772",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_03_16,LEG_03_23"
  },
  {
    "id": 773,
    "start_hour": 49,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0773",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_03_173,LEG_03_174"
  },
  {
    "id": 774,
    "start_hour": 49,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0774",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_03_78,LEG_03_251"
  },
  {
    "id": 775,
    "start_hour": 51,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0775",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_03_97,LEG_03_100"
  },
  {
    "id": 776,
    "start_hour": 36,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0776",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_03_74,LEG_03_70"
  },
  {
    "id": 777,
    "start_hour": 48,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0777",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_03_253,LEG_03_254"
  },
  {
    "id": 778,
    "start_hour": 39,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0778",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_03_154,LEG_03_187"
  },
  {
    "id": 779,
    "start_hour": 39,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0779",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_03_36,LEG_03_221"
  },
  {
    "id": 780,
    "start_hour": 40,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0780",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_03_116,LEG_03_12"
  },
  {
    "id": 781,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0781",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_03_257,LEG_03_60"
  },
  {
    "id": 782,
    "start_hour": 68,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0782",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_04_86,LEG_04_130,LEG_04_137"
  },
  {
    "id": 783,
    "start_hour": 52,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0783",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_03_77,LEG_03_121"
  },
  {
    "id": 784,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0784",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_04_124,LEG_04_76"
  },
  {
    "id": 785,
    "start_hour": 56,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0785",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_03_252,LEG_03_1"
  },
  {
    "id": 786,
    "start_hour": 55,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0786",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_03_114,LEG_03_117"
  },
  {
    "id": 787,
    "start_hour": 55,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0787",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_03_149,LEG_03_153"
  },
  {
    "id": 788,
    "start_hour": 52,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0788",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_03_93,LEG_03_92"
  },
  {
    "id": 789,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0789",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_03_34,LEG_03_37"
  },
  {
    "id": 790,
    "start_hour": 53,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0790",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_03_19,LEG_03_26"
  },
  {
    "id": 791,
    "start_hour": 40,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0791",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_03_2,LEG_03_62"
  },
  {
    "id": 792,
    "start_hour": 61,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0792",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_04_195,LEG_04_125"
  },
  {
    "id": 793,
    "start_hour": 86,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0793",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_05_51,LEG_05_84"
  },
  {
    "id": 794,
    "start_hour": 48,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0794",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_03_141,LEG_03_214,LEG_03_44"
  },
  {
    "id": 795,
    "start_hour": 71,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0795",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_04_216,LEG_04_67"
  },
  {
    "id": 796,
    "start_hour": 90,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0796",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_05_55,LEG_05_167"
  },
  {
    "id": 797,
    "start_hour": 36,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0797",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_03_150,LEG_03_204,LEG_03_205"
  },
  {
    "id": 798,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0798",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_04_72,LEG_04_155,LEG_04_47"
  },
  {
    "id": 799,
    "start_hour": 95,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0799",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_05_171,LEG_05_202,LEG_05_130"
  },
  {
    "id": 800,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0800",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_03_159,LEG_03_103"
  },
  {
    "id": 801,
    "start_hour": 63,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0801",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_04_136,LEG_04_228,LEG_04_65"
  },
  {
    "id": 802,
    "start_hour": 98,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0802",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_05_16"
  },
  {
    "id": 803,
    "start_hour": 59,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0803",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_03_32"
  },
  {
    "id": 804,
    "start_hour": 83,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0804",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_04_37"
  },
  {
    "id": 805,
    "start_hour": 57,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0805",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_03_163,LEG_03_220"
  },
  {
    "id": 806,
    "start_hour": 62,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0806",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_04_102,LEG_04_95,LEG_04_7"
  },
  {
    "id": 807,
    "start_hour": 100,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0807",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_05_37,LEG_05_199,LEG_05_183"
  },
  {
    "id": 808,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0808",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_06_110,LEG_06_66"
  },
  {
    "id": 809,
    "start_hour": 57,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0809",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_03_94,LEG_03_179"
  },
  {
    "id": 810,
    "start_hour": 62,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0810",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_04_235"
  },
  {
    "id": 811,
    "start_hour": 98,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0811",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_05_210,LEG_05_86,LEG_05_197,LEG_05_208"
  },
  {
    "id": 812,
    "start_hour": 120,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0812",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_06_222,LEG_06_116,LEG_06_119"
  },
  {
    "id": 813,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0813",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_03_45,LEG_03_171,LEG_03_223"
  },
  {
    "id": 814,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0814",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_04_183,LEG_04_145"
  },
  {
    "id": 815,
    "start_hour": 99,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0815",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_05_133,LEG_05_18"
  },
  {
    "id": 816,
    "start_hour": 119,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0816",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_06_86,LEG_06_202,LEG_06_201"
  },
  {
    "id": 817,
    "start_hour": 53,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0817",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_03_168,LEG_03_169"
  },
  {
    "id": 818,
    "start_hour": 60,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D0818",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_04_29,LEG_04_69,LEG_04_68"
  },
  {
    "id": 819,
    "start_hour": 100,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0819",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_05_19,LEG_05_120"
  },
  {
    "id": 820,
    "start_hour": 109,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0820",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_06_117"
  },
  {
    "id": 821,
    "start_hour": 50,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0821",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_03_188,LEG_03_87"
  },
  {
    "id": 822,
    "start_hour": 81,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0822",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_04_58"
  },
  {
    "id": 823,
    "start_hour": 87,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D0823",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_05_190,LEG_05_5"
  },
  {
    "id": 824,
    "start_hour": 108,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0824",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_06_134,LEG_06_188,LEG_06_185,LEG_06_95"
  },
  {
    "id": 825,
    "start_hour": 655,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0825",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_28_90,LEG_28_125"
  },
  {
    "id": 826,
    "start_hour": 661,
    "duration_hours": 28,
    "required_skill": "A319",
    "gerad_duty_id": "D0826",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_29_100,LEG_29_188,LEG_29_257"
  },
  {
    "id": 827,
    "start_hour": 704,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0827",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_30_61"
  },
  {
    "id": 828,
    "start_hour": 709,
    "duration_hours": 28,
    "required_skill": "A319",
    "gerad_duty_id": "D0828",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_31_58,LEG_31_65"
  },
  {
    "id": 829,
    "start_hour": 658,
    "duration_hours": 2,
    "required_skill": "A321",
    "gerad_duty_id": "D0829",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_28_208"
  },
  {
    "id": 830,
    "start_hour": 660,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0830",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_29_237,LEG_29_39,LEG_29_35,LEG_29_104"
  },
  {
    "id": 831,
    "start_hour": 684,
    "duration_hours": 2,
    "required_skill": "A321",
    "gerad_duty_id": "D0831",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_30_40"
  },
  {
    "id": 832,
    "start_hour": 637,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D0832",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_28_49,LEG_28_20,LEG_28_93"
  },
  {
    "id": 833,
    "start_hour": 684,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0833",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_29_29"
  },
  {
    "id": 834,
    "start_hour": 688,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0834",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_30_176"
  },
  {
    "id": 835,
    "start_hour": 636,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0835",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_28_78,LEG_28_96,LEG_28_249"
  },
  {
    "id": 836,
    "start_hour": 672,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0836",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_29_82,LEG_29_269"
  },
  {
    "id": 837,
    "start_hour": 691,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0837",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_30_69,LEG_30_68,LEG_30_43"
  },
  {
    "id": 838,
    "start_hour": 654,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0838",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_28_38,LEG_28_42"
  },
  {
    "id": 839,
    "start_hour": 662,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0839",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_29_230,LEG_29_136,LEG_29_54"
  },
  {
    "id": 840,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0840",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_28_106,LEG_28_99"
  },
  {
    "id": 841,
    "start_hour": 664,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0841",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_29_121,LEG_29_220,LEG_29_36,LEG_29_50"
  },
  {
    "id": 842,
    "start_hour": 650,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0842",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_28_218,LEG_28_224"
  },
  {
    "id": 843,
    "start_hour": 638,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0843",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_28_41,LEG_28_48"
  },
  {
    "id": 844,
    "start_hour": 638,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0844",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_28_34,LEG_28_217"
  },
  {
    "id": 845,
    "start_hour": 648,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0845",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_28_216,LEG_28_215,LEG_28_220,LEG_28_79"
  },
  {
    "id": 846,
    "start_hour": 648,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0846",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_28_207,LEG_28_263"
  },
  {
    "id": 847,
    "start_hour": 648,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0847",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_28_107,LEG_28_109"
  },
  {
    "id": 848,
    "start_hour": 636,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0848",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_28_184,LEG_28_183,LEG_28_76"
  },
  {
    "id": 849,
    "start_hour": 661,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0849",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_29_252"
  },
  {
    "id": 850,
    "start_hour": 658,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0850",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_28_104"
  },
  {
    "id": 851,
    "start_hour": 660,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0851",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_29_40,LEG_29_213,LEG_29_210"
  },
  {
    "id": 852,
    "start_hour": 653,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0852",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_28_210,LEG_28_124"
  },
  {
    "id": 853,
    "start_hour": 661,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0853",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_29_17,LEG_29_21,LEG_29_143"
  },
  {
    "id": 854,
    "start_hour": 697,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0854",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_30_211"
  },
  {
    "id": 855,
    "start_hour": 650,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0855",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_28_241"
  },
  {
    "id": 856,
    "start_hour": 680,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0856",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_29_243"
  },
  {
    "id": 857,
    "start_hour": 686,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0857",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_30_229,LEG_30_136,LEG_30_54"
  },
  {
    "id": 858,
    "start_hour": 652,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0858",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_28_102,LEG_28_92"
  },
  {
    "id": 859,
    "start_hour": 661,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D0859",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_29_157,LEG_29_194,LEG_29_207"
  },
  {
    "id": 860,
    "start_hour": 702,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0860",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_30_204,LEG_30_14"
  },
  {
    "id": 861,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0861",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_31_162,LEG_31_259,LEG_31_186"
  },
  {
    "id": 862,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0862",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_23_15,LEG_23_22"
  },
  {
    "id": 863,
    "start_hour": 520,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0863",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_23_115,LEG_23_12"
  },
  {
    "id": 864,
    "start_hour": 516,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0864",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_23_74,LEG_23_70"
  },
  {
    "id": 865,
    "start_hour": 519,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0865",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_23_150,LEG_23_183"
  },
  {
    "id": 866,
    "start_hour": 519,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0866",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_23_35,LEG_23_216"
  },
  {
    "id": 867,
    "start_hour": 528,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0867",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_23_251,LEG_23_252"
  },
  {
    "id": 868,
    "start_hour": 531,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0868",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_23_96,LEG_23_99"
  },
  {
    "id": 869,
    "start_hour": 529,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0869",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_23_78,LEG_23_249"
  },
  {
    "id": 870,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0870",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_23_44,LEG_23_168,LEG_23_223"
  },
  {
    "id": 871,
    "start_hour": 554,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0871",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_24_179,LEG_24_180,LEG_24_6"
  },
  {
    "id": 872,
    "start_hour": 528,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0872",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_23_138,LEG_23_210,LEG_23_43"
  },
  {
    "id": 873,
    "start_hour": 551,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0873",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_24_212,LEG_24_221,LEG_24_170"
  },
  {
    "id": 874,
    "start_hour": 533,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0874",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_23_152,LEG_23_147,LEG_23_100"
  },
  {
    "id": 875,
    "start_hour": 552,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0875",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_24_26,LEG_24_130,LEG_24_133"
  },
  {
    "id": 876,
    "start_hour": 527,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0876",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_23_255,LEG_23_59"
  },
  {
    "id": 877,
    "start_hour": 548,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0877",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_24_85,LEG_24_129,LEG_24_134"
  },
  {
    "id": 878,
    "start_hour": 536,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0878",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_23_19,LEG_23_196"
  },
  {
    "id": 879,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0879",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_24_207,LEG_24_8"
  },
  {
    "id": 880,
    "start_hour": 539,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0880",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_23_7"
  },
  {
    "id": 881,
    "start_hour": 554,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0881",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_24_5"
  },
  {
    "id": 882,
    "start_hour": 532,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0882",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_23_92,LEG_23_91"
  },
  {
    "id": 883,
    "start_hour": 535,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0883",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_23_145,LEG_23_149"
  },
  {
    "id": 884,
    "start_hour": 535,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0884",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_23_113,LEG_23_116"
  },
  {
    "id": 885,
    "start_hour": 536,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0885",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_23_250,LEG_23_1"
  },
  {
    "id": 886,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0886",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_23_155,LEG_23_102"
  },
  {
    "id": 887,
    "start_hour": 533,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0887",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_23_18,LEG_23_25"
  },
  {
    "id": 888,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0888",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_23_130,LEG_23_133,LEG_23_253"
  },
  {
    "id": 889,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0889",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_24_154,LEG_24_73,LEG_24_156"
  },
  {
    "id": 890,
    "start_hour": 529,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0890",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_23_182,LEG_23_83,LEG_23_188"
  },
  {
    "id": 891,
    "start_hour": 553,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0891",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_24_40,LEG_24_86"
  },
  {
    "id": 892,
    "start_hour": 584,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0892",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_25_88,LEG_25_231"
  },
  {
    "id": 893,
    "start_hour": 600,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0893",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_26_216,LEG_26_118,LEG_26_121"
  },
  {
    "id": 894,
    "start_hour": 533,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0894",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_23_164,LEG_23_165"
  },
  {
    "id": 895,
    "start_hour": 540,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D0895",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_24_28,LEG_24_69,LEG_24_68"
  },
  {
    "id": 896,
    "start_hour": 580,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0896",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_25_20,LEG_25_132"
  },
  {
    "id": 897,
    "start_hour": 589,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0897",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_26_119"
  },
  {
    "id": 898,
    "start_hour": 539,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0898",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_23_107"
  },
  {
    "id": 899,
    "start_hour": 554,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0899",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_24_98,LEG_24_97,LEG_24_213,LEG_24_100"
  },
  {
    "id": 900,
    "start_hour": 576,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0900",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_25_26,LEG_25_11"
  },
  {
    "id": 901,
    "start_hour": 596,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0901",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_26_9,LEG_26_64,LEG_26_124"
  },
  {
    "id": 902,
    "start_hour": 539,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0902",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_23_21"
  },
  {
    "id": 903,
    "start_hour": 556,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0903",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_24_64,LEG_24_188"
  },
  {
    "id": 904,
    "start_hour": 577,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0904",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_25_40,LEG_25_41"
  },
  {
    "id": 905,
    "start_hour": 602,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0905",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_26_29"
  },
  {
    "id": 906,
    "start_hour": 537,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0906",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_23_159,LEG_23_214"
  },
  {
    "id": 907,
    "start_hour": 542,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0907",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_24_101,LEG_24_94,LEG_24_253"
  },
  {
    "id": 908,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0908",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_25_154,LEG_25_73,LEG_25_156"
  },
  {
    "id": 909,
    "start_hour": 539,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0909",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_23_31"
  },
  {
    "id": 910,
    "start_hour": 560,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0910",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_24_29,LEG_24_81"
  },
  {
    "id": 911,
    "start_hour": 579,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0911",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_25_194,LEG_25_197,LEG_25_104"
  },
  {
    "id": 912,
    "start_hour": 537,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0912",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_23_141"
  },
  {
    "id": 913,
    "start_hour": 555,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0913",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_24_143,LEG_24_142"
  },
  {
    "id": 914,
    "start_hour": 581,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0914",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_25_208,LEG_25_127"
  },
  {
    "id": 915,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0915",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_26_180,LEG_26_179,LEG_26_92"
  },
  {
    "id": 916,
    "start_hour": 529,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0916",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_23_169,LEG_23_220,LEG_23_63"
  },
  {
    "id": 917,
    "start_hour": 550,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0917",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_24_225,LEG_24_187"
  },
  {
    "id": 918,
    "start_hour": 580,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0918",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_25_189,LEG_25_188"
  },
  {
    "id": 919,
    "start_hour": 601,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0919",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_26_36,LEG_26_107,LEG_26_104"
  },
  {
    "id": 920,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0920",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_12_17,LEG_12_23"
  },
  {
    "id": 921,
    "start_hour": 265,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0921",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_12_69,LEG_12_221"
  },
  {
    "id": 922,
    "start_hour": 265,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0922",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_12_157,LEG_12_158"
  },
  {
    "id": 923,
    "start_hour": 256,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0923",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_12_107,LEG_12_11"
  },
  {
    "id": 924,
    "start_hour": 267,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0924",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_12_88,LEG_12_91"
  },
  {
    "id": 925,
    "start_hour": 255,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0925",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_12_140,LEG_12_167"
  },
  {
    "id": 926,
    "start_hour": 264,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0926",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_12_223,LEG_12_224"
  },
  {
    "id": 927,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0927",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_12_193,LEG_12_194"
  },
  {
    "id": 928,
    "start_hour": 255,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0928",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_12_34,LEG_12_196"
  },
  {
    "id": 929,
    "start_hour": 263,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0929",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_12_227,LEG_12_53"
  },
  {
    "id": 930,
    "start_hour": 284,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0930",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_13_79,LEG_13_206,LEG_13_204"
  },
  {
    "id": 931,
    "start_hour": 269,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0931",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_12_143,LEG_12_137,LEG_12_205"
  },
  {
    "id": 932,
    "start_hour": 288,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0932",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_13_227,LEG_13_118,LEG_13_130"
  },
  {
    "id": 933,
    "start_hour": 252,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0933",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_12_67,LEG_12_116,LEG_12_228"
  },
  {
    "id": 934,
    "start_hour": 277,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0934",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_13_176"
  },
  {
    "id": 935,
    "start_hour": 268,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0935",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_12_84,LEG_12_83"
  },
  {
    "id": 936,
    "start_hour": 272,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0936",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_12_222,LEG_12_0"
  },
  {
    "id": 937,
    "start_hour": 269,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0937",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_12_19,LEG_12_25"
  },
  {
    "id": 938,
    "start_hour": 271,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0938",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_12_135,LEG_12_139"
  },
  {
    "id": 939,
    "start_hour": 254,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0939",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_12_33,LEG_12_29,LEG_12_68"
  },
  {
    "id": 940,
    "start_hour": 292,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0940",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_13_78,LEG_13_77"
  },
  {
    "id": 941,
    "start_hour": 314,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0941",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_14_72,LEG_14_181,LEG_14_7"
  },
  {
    "id": 942,
    "start_hour": 275,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0942",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_12_31"
  },
  {
    "id": 943,
    "start_hour": 296,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0943",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_13_26,LEG_13_69"
  },
  {
    "id": 944,
    "start_hour": 313,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0944",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_14_19,LEG_14_136,LEG_14_237"
  },
  {
    "id": 945,
    "start_hour": 256,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0945",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_12_1,LEG_12_55"
  },
  {
    "id": 946,
    "start_hour": 277,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0946",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_13_183,LEG_13_113"
  },
  {
    "id": 947,
    "start_hour": 302,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0947",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_14_58,LEG_14_94"
  },
  {
    "id": 948,
    "start_hour": 273,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0948",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_12_85,LEG_12_162"
  },
  {
    "id": 949,
    "start_hour": 278,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0949",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_13_96,LEG_13_89,LEG_13_6"
  },
  {
    "id": 950,
    "start_hour": 314,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0950",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_14_6,LEG_14_152"
  },
  {
    "id": 951,
    "start_hour": 336,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0951",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_15_158,LEG_15_130,LEG_15_133"
  },
  {
    "id": 952,
    "start_hour": 275,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0952",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_12_12"
  },
  {
    "id": 953,
    "start_hour": 282,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0953",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_13_56,LEG_13_185"
  },
  {
    "id": 954,
    "start_hour": 316,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0954",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_14_190,LEG_14_103"
  },
  {
    "id": 955,
    "start_hour": 334,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0955",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_15_225,LEG_15_226,LEG_15_140"
  },
  {
    "id": 956,
    "start_hour": 269,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0956",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_12_154,LEG_12_155"
  },
  {
    "id": 957,
    "start_hour": 276,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0957",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_13_202,LEG_13_58,LEG_13_103"
  },
  {
    "id": 958,
    "start_hour": 312,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0958",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_14_110,LEG_14_42"
  },
  {
    "id": 959,
    "start_hour": 338,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0959",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_15_32"
  },
  {
    "id": 960,
    "start_hour": 252,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0960",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_12_136,LEG_12_183,LEG_12_184"
  },
  {
    "id": 961,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0961",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_13_66,LEG_13_131"
  },
  {
    "id": 962,
    "start_hour": 315,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0962",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_14_144,LEG_14_131"
  },
  {
    "id": 963,
    "start_hour": 325,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0963",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_15_131"
  },
  {
    "id": 964,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0964",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_12_120,LEG_12_123,LEG_12_132"
  },
  {
    "id": 965,
    "start_hour": 293,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0965",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_13_199,LEG_13_116"
  },
  {
    "id": 966,
    "start_hour": 314,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0966",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_14_199,LEG_14_202"
  },
  {
    "id": 967,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0967",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_15_72,LEG_15_241,LEG_15_34"
  },
  {
    "id": 968,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0968",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_12_3,LEG_12_124"
  },
  {
    "id": 969,
    "start_hour": 278,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0969",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_13_165,LEG_13_167"
  },
  {
    "id": 970,
    "start_hour": 300,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0970",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_14_147,LEG_14_201,LEG_14_196,LEG_14_193,LEG_14_46"
  },
  {
    "id": 971,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0971",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_15_163,LEG_15_4,LEG_15_215"
  },
  {
    "id": 972,
    "start_hour": 577,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0972",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_25_211,LEG_25_210"
  },
  {
    "id": 973,
    "start_hour": 575,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0973",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_25_51,LEG_25_52"
  },
  {
    "id": 974,
    "start_hour": 581,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0974",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_25_56,LEG_25_245"
  },
  {
    "id": 975,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0975",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_26_226,LEG_26_46"
  },
  {
    "id": 976,
    "start_hour": 587,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0976",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_25_193"
  },
  {
    "id": 977,
    "start_hour": 604,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0977",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_26_172"
  },
  {
    "id": 978,
    "start_hour": 587,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0978",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_25_37"
  },
  {
    "id": 979,
    "start_hour": 604,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0979",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_26_37"
  },
  {
    "id": 980,
    "start_hour": 579,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0980",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_25_60,LEG_25_89,LEG_25_209,LEG_25_84"
  },
  {
    "id": 981,
    "start_hour": 584,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0981",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_25_82"
  },
  {
    "id": 982,
    "start_hour": 602,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0982",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_26_110,LEG_26_115"
  },
  {
    "id": 983,
    "start_hour": 628,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0983",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_27_92,LEG_27_33,LEG_27_41"
  },
  {
    "id": 984,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0984",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_25_221,LEG_25_170,LEG_25_177"
  },
  {
    "id": 985,
    "start_hour": 600,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0985",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_26_156,LEG_26_24"
  },
  {
    "id": 986,
    "start_hour": 619,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0986",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_27_29"
  },
  {
    "id": 987,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0987",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_25_119,LEG_25_114"
  },
  {
    "id": 988,
    "start_hour": 590,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0988",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_26_145,LEG_26_147,LEG_26_128"
  },
  {
    "id": 989,
    "start_hour": 625,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0989",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_27_173,LEG_27_73,LEG_27_154"
  },
  {
    "id": 990,
    "start_hour": 580,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0990",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_25_230,LEG_25_135,LEG_25_75"
  },
  {
    "id": 991,
    "start_hour": 604,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0991",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_26_73,LEG_26_201"
  },
  {
    "id": 992,
    "start_hour": 627,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0992",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_27_146,LEG_27_100"
  },
  {
    "id": 993,
    "start_hour": 585,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0993",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_25_43,LEG_25_34"
  },
  {
    "id": 994,
    "start_hour": 590,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0994",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_26_30,LEG_26_26,LEG_26_68"
  },
  {
    "id": 995,
    "start_hour": 626,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0995",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_27_16,LEG_27_114"
  },
  {
    "id": 996,
    "start_hour": 649,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0996",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_28_211"
  },
  {
    "id": 997,
    "start_hour": 582,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0997",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_25_49,LEG_25_107"
  },
  {
    "id": 998,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0998",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_26_87,LEG_26_132"
  },
  {
    "id": 999,
    "start_hour": 624,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0999",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_27_164,LEG_27_163"
  },
  {
    "id": 1000,
    "start_hour": 643,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1000",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_28_69,LEG_28_235,LEG_28_213"
  },
  {
    "id": 1001,
    "start_hour": 584,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1001",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_25_223"
  },
  {
    "id": 1002,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1002",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_26_163,LEG_26_129"
  },
  {
    "id": 1003,
    "start_hour": 630,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1003",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_27_168,LEG_27_12"
  },
  {
    "id": 1004,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1004",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_28_161,LEG_28_257,LEG_28_185"
  },
  {
    "id": 1005,
    "start_hour": 585,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1005",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_25_219,LEG_25_111"
  },
  {
    "id": 1006,
    "start_hour": 589,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1006",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_26_196,LEG_26_197,LEG_26_42,LEG_26_27"
  },
  {
    "id": 1007,
    "start_hour": 612,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1007",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_27_6,LEG_27_108,LEG_27_118,LEG_27_201"
  },
  {
    "id": 1008,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1008",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_25_167,LEG_25_23"
  },
  {
    "id": 1009,
    "start_hour": 590,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1009",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_26_161,LEG_26_162"
  },
  {
    "id": 1010,
    "start_hour": 612,
    "duration_hours": 27,
    "required_skill": "A321",
    "gerad_duty_id": "D1010",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_27_131,LEG_27_22"
  },
  {
    "id": 1011,
    "start_hour": 639,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1011",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_28_26,LEG_28_54"
  },
  {
    "id": 1012,
    "start_hour": 465,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1012",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_20_217"
  },
  {
    "id": 1013,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1013",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_20_236,LEG_20_238"
  },
  {
    "id": 1014,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1014",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_20_190,LEG_20_189"
  },
  {
    "id": 1015,
    "start_hour": 459,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1015",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_20_216"
  },
  {
    "id": 1016,
    "start_hour": 446,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1016",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_20_112,LEG_20_71"
  },
  {
    "id": 1017,
    "start_hour": 469,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1017",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_21_219,LEG_21_220,LEG_21_83"
  },
  {
    "id": 1018,
    "start_hour": 506,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1018",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_22_122"
  },
  {
    "id": 1019,
    "start_hour": 464,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1019",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_20_84,LEG_20_213"
  },
  {
    "id": 1020,
    "start_hour": 480,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1020",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_21_240,LEG_21_175"
  },
  {
    "id": 1021,
    "start_hour": 504,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1021",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_22_174,LEG_22_144,LEG_22_203"
  },
  {
    "id": 1022,
    "start_hour": 467,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1022",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_20_68"
  },
  {
    "id": 1023,
    "start_hour": 485,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1023",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_21_56,LEG_21_247"
  },
  {
    "id": 1024,
    "start_hour": 466,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1024",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_20_219"
  },
  {
    "id": 1025,
    "start_hour": 469,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1025",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_21_114,LEG_21_108,LEG_21_109"
  },
  {
    "id": 1026,
    "start_hour": 506,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1026",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_22_98,LEG_22_151"
  },
  {
    "id": 1027,
    "start_hour": 528,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1027",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_23_158,LEG_23_144,LEG_23_203"
  },
  {
    "id": 1028,
    "start_hour": 467,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1028",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_20_118"
  },
  {
    "id": 1029,
    "start_hour": 491,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1029",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_21_80"
  },
  {
    "id": 1030,
    "start_hour": 509,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1030",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_22_56,LEG_22_245"
  },
  {
    "id": 1031,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1031",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_20_192,LEG_20_7"
  },
  {
    "id": 1032,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1032",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_21_44,LEG_21_170,LEG_21_54"
  },
  {
    "id": 1033,
    "start_hour": 510,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1033",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_22_3,LEG_22_171,LEG_22_199"
  },
  {
    "id": 1034,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1034",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_20_187,LEG_20_137"
  },
  {
    "id": 1035,
    "start_hour": 470,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1035",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_21_107,LEG_21_163,LEG_21_99,LEG_21_215"
  },
  {
    "id": 1036,
    "start_hour": 492,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D1036",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_22_28,LEG_22_69,LEG_22_68"
  },
  {
    "id": 1037,
    "start_hour": 532,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1037",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_23_20,LEG_23_171,LEG_23_199"
  },
  {
    "id": 1038,
    "start_hour": 50,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1038",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_03_256,LEG_03_259"
  },
  {
    "id": 1039,
    "start_hour": 50,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1039",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_03_209,LEG_03_208"
  },
  {
    "id": 1040,
    "start_hour": 59,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1040",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_03_129"
  },
  {
    "id": 1041,
    "start_hour": 78,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1041",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_04_127,LEG_04_129"
  },
  {
    "id": 1042,
    "start_hour": 98,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1042",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_05_110"
  },
  {
    "id": 1043,
    "start_hour": 58,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1043",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_03_238"
  },
  {
    "id": 1044,
    "start_hour": 61,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1044",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_04_49,LEG_04_51,LEG_04_50,LEG_04_31,LEG_04_232"
  },
  {
    "id": 1045,
    "start_hour": 96,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1045",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_05_213,LEG_05_160"
  },
  {
    "id": 1046,
    "start_hour": 120,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1046",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_06_161,LEG_06_132,LEG_06_191"
  },
  {
    "id": 1047,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1047",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_03_206,LEG_03_152"
  },
  {
    "id": 1048,
    "start_hour": 62,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1048",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_04_180,LEG_04_182"
  },
  {
    "id": 1049,
    "start_hour": 84,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1049",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_05_136,LEG_05_184,LEG_05_185"
  },
  {
    "id": 1050,
    "start_hour": 122,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1050",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_06_109"
  },
  {
    "id": 1051,
    "start_hour": 631,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1051",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_27_215,LEG_27_98"
  },
  {
    "id": 1052,
    "start_hour": 637,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1052",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_28_136,LEG_28_248,LEG_28_158"
  },
  {
    "id": 1053,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1053",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_29_160,LEG_29_97"
  },
  {
    "id": 1054,
    "start_hour": 684,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1054",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_30_192"
  },
  {
    "id": 1055,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1055",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_27_11,LEG_27_51"
  },
  {
    "id": 1056,
    "start_hour": 648,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1056",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_28_254,LEG_28_198"
  },
  {
    "id": 1057,
    "start_hour": 667,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1057",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_29_69,LEG_29_236,LEG_29_214"
  },
  {
    "id": 1058,
    "start_hour": 686,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1058",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_30_91,LEG_30_199"
  },
  {
    "id": 1059,
    "start_hour": 633,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1059",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_27_8"
  },
  {
    "id": 1060,
    "start_hour": 637,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1060",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_28_156,LEG_28_193,LEG_28_206"
  },
  {
    "id": 1061,
    "start_hour": 678,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1061",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_29_205"
  },
  {
    "id": 1062,
    "start_hour": 636,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1062",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_27_54"
  },
  {
    "id": 1063,
    "start_hour": 654,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1063",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_28_44,LEG_28_251"
  },
  {
    "id": 1064,
    "start_hour": 614,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1064",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_27_95,LEG_27_202"
  },
  {
    "id": 1065,
    "start_hour": 627,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1065",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_27_90,LEG_27_89"
  },
  {
    "id": 1066,
    "start_hour": 627,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1066",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_27_171"
  },
  {
    "id": 1067,
    "start_hour": 634,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1067",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_27_198"
  },
  {
    "id": 1068,
    "start_hour": 627,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1068",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_27_211,LEG_27_214"
  },
  {
    "id": 1069,
    "start_hour": 634,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1069",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_27_39"
  },
  {
    "id": 1070,
    "start_hour": 636,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1070",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_27_213"
  },
  {
    "id": 1071,
    "start_hour": 656,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1071",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_28_61"
  },
  {
    "id": 1072,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1072",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_29_5,LEG_29_67,LEG_29_76"
  },
  {
    "id": 1073,
    "start_hour": 685,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1073",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_30_251,LEG_30_44,LEG_30_252"
  },
  {
    "id": 1074,
    "start_hour": 628,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1074",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_27_199"
  },
  {
    "id": 1075,
    "start_hour": 656,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1075",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_28_242"
  },
  {
    "id": 1076,
    "start_hour": 662,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D1076",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_29_53,LEG_29_178"
  },
  {
    "id": 1077,
    "start_hour": 685,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1077",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_30_156,LEG_30_193,LEG_30_246,LEG_30_243"
  },
  {
    "id": 1078,
    "start_hour": 631,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1078",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_27_162,LEG_27_117"
  },
  {
    "id": 1079,
    "start_hour": 640,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1079",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_28_232,LEG_28_230,LEG_28_86"
  },
  {
    "id": 1080,
    "start_hour": 675,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1080",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_29_85,LEG_29_172"
  },
  {
    "id": 1081,
    "start_hour": 697,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1081",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_30_173,LEG_30_148,LEG_30_203"
  },
  {
    "id": 1082,
    "start_hour": 297,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1082",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_13_223"
  },
  {
    "id": 1083,
    "start_hour": 290,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1083",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_13_243,LEG_13_246"
  },
  {
    "id": 1084,
    "start_hour": 290,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1084",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_13_196,LEG_13_195"
  },
  {
    "id": 1085,
    "start_hour": 299,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1085",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_13_117"
  },
  {
    "id": 1086,
    "start_hour": 314,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1086",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_14_122"
  },
  {
    "id": 1087,
    "start_hour": 279,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1087",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_13_197,LEG_13_98"
  },
  {
    "id": 1088,
    "start_hour": 301,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1088",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_14_192,LEG_14_124"
  },
  {
    "id": 1089,
    "start_hour": 291,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1089",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_13_222"
  },
  {
    "id": 1090,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1090",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_13_193,LEG_13_138"
  },
  {
    "id": 1091,
    "start_hour": 302,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1091",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_14_177,LEG_14_179"
  },
  {
    "id": 1092,
    "start_hour": 324,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1092",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_15_146,LEG_15_200,LEG_15_201"
  },
  {
    "id": 1093,
    "start_hour": 362,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1093",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_16_122"
  },
  {
    "id": 1094,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1094",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_13_55,LEG_13_95"
  },
  {
    "id": 1095,
    "start_hour": 311,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1095",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_14_214,LEG_14_188"
  },
  {
    "id": 1096,
    "start_hour": 344,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1096",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_15_190"
  },
  {
    "id": 1097,
    "start_hour": 349,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1097",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_16_13,LEG_16_172,LEG_16_144,LEG_16_203"
  },
  {
    "id": 1098,
    "start_hour": 298,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1098",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_13_225"
  },
  {
    "id": 1099,
    "start_hour": 301,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1099",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_14_112,LEG_14_104"
  },
  {
    "id": 1100,
    "start_hour": 326,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1100",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_15_234"
  },
  {
    "id": 1101,
    "start_hour": 362,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1101",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_16_233,LEG_16_19,LEG_16_196"
  },
  {
    "id": 1102,
    "start_hour": 25,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1102",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_02_191"
  },
  {
    "id": 1103,
    "start_hour": 32,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1103",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_02_91"
  },
  {
    "id": 1104,
    "start_hour": 57,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1104",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_03_58"
  },
  {
    "id": 1105,
    "start_hour": 63,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1105",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_04_210,LEG_04_6,LEG_04_108"
  },
  {
    "id": 1106,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1106",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_05_87,LEG_05_141,LEG_05_61"
  },
  {
    "id": 1107,
    "start_hour": 32,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1107",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_02_223"
  },
  {
    "id": 1108,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1108",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_03_183,LEG_03_146"
  },
  {
    "id": 1109,
    "start_hour": 77,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1109",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_04_212,LEG_04_128"
  },
  {
    "id": 1110,
    "start_hour": 98,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1110",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_05_182,LEG_05_180,LEG_05_177"
  },
  {
    "id": 1111,
    "start_hour": 30,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1111",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_02_172,LEG_02_80,LEG_02_82"
  },
  {
    "id": 1112,
    "start_hour": 51,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1112",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_03_198,LEG_03_199,LEG_03_196,LEG_03_47"
  },
  {
    "id": 1113,
    "start_hour": 71,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1113",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_04_96,LEG_04_28"
  },
  {
    "id": 1114,
    "start_hour": 90,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1114",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_05_35"
  },
  {
    "id": 1115,
    "start_hour": 25,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1115",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_02_218,LEG_02_31,LEG_02_0"
  },
  {
    "id": 1116,
    "start_hour": 46,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1116",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_03_226,LEG_03_110"
  },
  {
    "id": 1117,
    "start_hour": 72,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1117",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_04_111,LEG_04_87"
  },
  {
    "id": 1118,
    "start_hour": 104,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1118",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_05_92"
  },
  {
    "id": 1119,
    "start_hour": 13,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1119",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_02_195,LEG_02_125"
  },
  {
    "id": 1120,
    "start_hour": 38,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1120",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_03_59,LEG_03_96"
  },
  {
    "id": 1121,
    "start_hour": 64,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1121",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_04_2,LEG_04_62"
  },
  {
    "id": 1122,
    "start_hour": 33,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1122",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_02_13,LEG_02_112"
  },
  {
    "id": 1123,
    "start_hour": 37,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1123",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_03_113,LEG_03_107,LEG_03_185"
  },
  {
    "id": 1124,
    "start_hour": 71,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1124",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_04_109,LEG_04_119,LEG_04_224"
  },
  {
    "id": 1125,
    "start_hour": 23,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1125",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_02_81"
  },
  {
    "id": 1126,
    "start_hour": 45,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1126",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_03_126,LEG_03_248"
  },
  {
    "id": 1127,
    "start_hour": 66,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1127",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_04_56,LEG_04_233,LEG_04_118"
  },
  {
    "id": 1128,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1128",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_02_120,LEG_02_115"
  },
  {
    "id": 1129,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1129",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_03_161,LEG_03_164,LEG_03_108"
  },
  {
    "id": 1130,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1130",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_04_99,LEG_04_242,LEG_04_46"
  },
  {
    "id": 1131,
    "start_hour": 32,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1131",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_02_194"
  },
  {
    "id": 1132,
    "start_hour": 28,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1132",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_02_231,LEG_02_229"
  },
  {
    "id": 1133,
    "start_hour": 23,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1133",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_02_222,LEG_02_230,LEG_02_50,LEG_02_46"
  },
  {
    "id": 1134,
    "start_hour": 23,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1134",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_02_52,LEG_02_53"
  },
  {
    "id": 1135,
    "start_hour": 13,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1135",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_02_48,LEG_02_190"
  },
  {
    "id": 1136,
    "start_hour": 27,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1136",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_02_61,LEG_02_90,LEG_02_213,LEG_02_85"
  },
  {
    "id": 1137,
    "start_hour": 128,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1137",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_06_209"
  },
  {
    "id": 1138,
    "start_hour": 121,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1138",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_06_175"
  },
  {
    "id": 1139,
    "start_hour": 125,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1139",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_06_77"
  },
  {
    "id": 1140,
    "start_hour": 152,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1140",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_07_89,LEG_07_75"
  },
  {
    "id": 1141,
    "start_hour": 169,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1141",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_08_18,LEG_08_139,LEG_08_237"
  },
  {
    "id": 1142,
    "start_hour": 191,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1142",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_09_104,LEG_09_250,LEG_09_54"
  },
  {
    "id": 1143,
    "start_hour": 124,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1143",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_06_213,LEG_06_211"
  },
  {
    "id": 1144,
    "start_hour": 133,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1144",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_07_14,LEG_07_176,LEG_07_241"
  },
  {
    "id": 1145,
    "start_hour": 170,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1145",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_08_243,LEG_08_166"
  },
  {
    "id": 1146,
    "start_hour": 191,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1146",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_09_15,LEG_09_188"
  },
  {
    "id": 1147,
    "start_hour": 121,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1147",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_06_58"
  },
  {
    "id": 1148,
    "start_hour": 146,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1148",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_07_17,LEG_07_166"
  },
  {
    "id": 1149,
    "start_hour": 167,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1149",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_08_15,LEG_08_28"
  },
  {
    "id": 1150,
    "start_hour": 186,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1150",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_09_40,LEG_09_222,LEG_09_230"
  },
  {
    "id": 1151,
    "start_hour": 126,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1151",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_06_155,LEG_06_70,LEG_06_65"
  },
  {
    "id": 1152,
    "start_hour": 145,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1152",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_07_18,LEG_07_139,LEG_07_238,LEG_07_181"
  },
  {
    "id": 1153,
    "start_hour": 167,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1153",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_08_249,LEG_08_67"
  },
  {
    "id": 1154,
    "start_hour": 186,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1154",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_09_63"
  },
  {
    "id": 1155,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1155",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_06_106,LEG_06_103"
  },
  {
    "id": 1156,
    "start_hour": 134,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1156",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_07_161,LEG_07_164,LEG_07_145"
  },
  {
    "id": 1157,
    "start_hour": 171,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1157",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_08_147,LEG_08_185"
  },
  {
    "id": 1158,
    "start_hour": 191,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1158",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_09_109,LEG_09_119,LEG_09_224"
  },
  {
    "id": 1159,
    "start_hour": 129,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1159",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_06_197,LEG_06_139"
  },
  {
    "id": 1160,
    "start_hour": 134,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1160",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_07_236"
  },
  {
    "id": 1161,
    "start_hour": 170,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1161",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_08_234,LEG_08_242,LEG_08_46"
  },
  {
    "id": 1162,
    "start_hour": 119,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1162",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_06_71"
  },
  {
    "id": 1163,
    "start_hour": 141,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1163",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_07_126,LEG_07_249"
  },
  {
    "id": 1164,
    "start_hour": 162,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1164",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_08_56,LEG_08_222,LEG_08_230"
  },
  {
    "id": 1165,
    "start_hour": 128,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1165",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_06_47"
  },
  {
    "id": 1166,
    "start_hour": 150,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1166",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_07_3,LEG_07_7"
  },
  {
    "id": 1167,
    "start_hour": 172,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1167",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_08_43"
  },
  {
    "id": 1168,
    "start_hour": 132,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1168",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_06_59"
  },
  {
    "id": 1169,
    "start_hour": 148,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1169",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_07_21,LEG_07_185"
  },
  {
    "id": 1170,
    "start_hour": 167,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1170",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_08_109,LEG_08_119,LEG_08_224"
  },
  {
    "id": 1171,
    "start_hour": 128,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1171",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_06_206"
  },
  {
    "id": 1172,
    "start_hour": 146,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1172",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_07_183,LEG_07_237,LEG_07_230"
  },
  {
    "id": 1173,
    "start_hour": 131,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1173",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_06_181"
  },
  {
    "id": 1174,
    "start_hour": 148,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1174",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_07_193"
  },
  {
    "id": 1175,
    "start_hour": 131,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1175",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_06_30"
  },
  {
    "id": 1176,
    "start_hour": 148,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1176",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_07_43"
  },
  {
    "id": 1177,
    "start_hour": 123,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1177",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_06_53,LEG_06_80,LEG_06_52"
  },
  {
    "id": 1178,
    "start_hour": 149,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1178",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_07_213,LEG_07_259"
  },
  {
    "id": 1179,
    "start_hour": 128,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1179",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_06_178"
  },
  {
    "id": 1180,
    "start_hour": 119,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1180",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_06_43,LEG_06_44"
  },
  {
    "id": 1181,
    "start_hour": 109,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1181",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_06_179,LEG_06_210"
  },
  {
    "id": 1182,
    "start_hour": 109,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1182",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_06_45,LEG_06_76"
  },
  {
    "id": 1183,
    "start_hour": 122,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1183",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_06_208"
  },
  {
    "id": 1184,
    "start_hour": 296,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1184",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_13_214"
  },
  {
    "id": 1185,
    "start_hour": 289,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1185",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_13_179"
  },
  {
    "id": 1186,
    "start_hour": 296,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1186",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_13_182"
  },
  {
    "id": 1187,
    "start_hour": 277,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1187",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_13_46,LEG_13_80"
  },
  {
    "id": 1188,
    "start_hour": 277,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1188",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_13_76,LEG_13_54"
  },
  {
    "id": 1189,
    "start_hour": 287,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1189",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_13_44,LEG_13_45"
  },
  {
    "id": 1190,
    "start_hour": 289,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1190",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_13_201,LEG_13_234,LEG_13_233,LEG_13_32"
  },
  {
    "id": 1191,
    "start_hour": 291,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1191",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_13_57,LEG_13_84,LEG_13_210"
  },
  {
    "id": 1192,
    "start_hour": 314,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1192",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_14_180,LEG_14_236,LEG_14_228"
  },
  {
    "id": 1193,
    "start_hour": 299,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1193",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_13_31"
  },
  {
    "id": 1194,
    "start_hour": 316,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1194",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_14_43"
  },
  {
    "id": 1195,
    "start_hour": 289,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1195",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_13_62"
  },
  {
    "id": 1196,
    "start_hour": 306,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1196",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_14_62"
  },
  {
    "id": 1197,
    "start_hour": 290,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1197",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_13_213"
  },
  {
    "id": 1198,
    "start_hour": 296,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1198",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_13_48"
  },
  {
    "id": 1199,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1199",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_14_0,LEG_14_182"
  },
  {
    "id": 1200,
    "start_hour": 335,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1200",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_15_108,LEG_15_118,LEG_15_224"
  },
  {
    "id": 1201,
    "start_hour": 300,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1201",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_13_50"
  },
  {
    "id": 1202,
    "start_hour": 318,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1202",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_14_4,LEG_14_23"
  },
  {
    "id": 1203,
    "start_hour": 340,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1203",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_15_64"
  },
  {
    "id": 1204,
    "start_hour": 287,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1204",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_13_74"
  },
  {
    "id": 1205,
    "start_hour": 309,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1205",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_14_125,LEG_14_249"
  },
  {
    "id": 1206,
    "start_hour": 330,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1206",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_15_55,LEG_15_232,LEG_15_117"
  },
  {
    "id": 1207,
    "start_hour": 293,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1207",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_13_81"
  },
  {
    "id": 1208,
    "start_hour": 308,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1208",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_14_86,LEG_14_71,LEG_14_137"
  },
  {
    "id": 1209,
    "start_hour": 328,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1209",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_15_2,LEG_15_61"
  },
  {
    "id": 1210,
    "start_hour": 299,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1210",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_13_30"
  },
  {
    "id": 1211,
    "start_hour": 314,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1211",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_14_29"
  },
  {
    "id": 1212,
    "start_hour": 330,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1212",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_15_39,LEG_15_222,LEG_15_229"
  },
  {
    "id": 1213,
    "start_hour": 297,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1213",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_13_200,LEG_13_141"
  },
  {
    "id": 1214,
    "start_hour": 302,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1214",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_14_244,LEG_14_241"
  },
  {
    "id": 1215,
    "start_hour": 324,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1215",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_15_9,LEG_15_153,LEG_15_77"
  },
  {
    "id": 1216,
    "start_hour": 356,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1216",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_16_70,LEG_16_182,LEG_16_83"
  },
  {
    "id": 1217,
    "start_hour": 296,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1217",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_13_85"
  },
  {
    "id": 1218,
    "start_hour": 320,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1218",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_14_88,LEG_14_75"
  },
  {
    "id": 1219,
    "start_hour": 337,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1219",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_15_17,LEG_15_136,LEG_15_236,LEG_15_171"
  },
  {
    "id": 1220,
    "start_hour": 359,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1220",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_16_103,LEG_16_248,LEG_16_53"
  },
  {
    "id": 1221,
    "start_hour": 300,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1221",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_13_29"
  },
  {
    "id": 1222,
    "start_hour": 302,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1222",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_14_105,LEG_14_162,LEG_14_96,LEG_14_216"
  },
  {
    "id": 1223,
    "start_hour": 324,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D1223",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_15_87,LEG_15_30,LEG_15_21"
  },
  {
    "id": 1224,
    "start_hour": 364,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1224",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_16_64"
  },
  {
    "id": 1225,
    "start_hour": 294,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1225",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_13_157,LEG_13_73,LEG_13_219"
  },
  {
    "id": 1226,
    "start_hour": 312,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1226",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_14_239,LEG_14_12"
  },
  {
    "id": 1227,
    "start_hour": 332,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1227",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_15_10,LEG_15_27"
  },
  {
    "id": 1228,
    "start_hour": 354,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1228",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_16_39,LEG_16_222,LEG_16_229"
  },
  {
    "id": 1229,
    "start_hour": 292,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1229",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_13_218,LEG_13_216"
  },
  {
    "id": 1230,
    "start_hour": 301,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D1230",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_14_15,LEG_14_173,LEG_14_240"
  },
  {
    "id": 1231,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1231",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_15_242,LEG_15_181"
  },
  {
    "id": 1232,
    "start_hour": 359,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1232",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_16_108,LEG_16_118,LEG_16_224"
  },
  {
    "id": 1233,
    "start_hour": 194,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1233",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_09_209,LEG_09_208"
  },
  {
    "id": 1234,
    "start_hour": 194,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1234",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_09_256,LEG_09_259"
  },
  {
    "id": 1235,
    "start_hour": 202,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1235",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_09_238"
  },
  {
    "id": 1236,
    "start_hour": 205,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1236",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_10_49,LEG_10_51,LEG_10_81"
  },
  {
    "id": 1237,
    "start_hour": 242,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1237",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_11_123"
  },
  {
    "id": 1238,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1238",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_09_206,LEG_09_152"
  },
  {
    "id": 1239,
    "start_hour": 206,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1239",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_10_176,LEG_10_178"
  },
  {
    "id": 1240,
    "start_hour": 228,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1240",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_11_148,LEG_11_202,LEG_11_203"
  },
  {
    "id": 1241,
    "start_hour": 266,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1241",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_12_112"
  },
  {
    "id": 1242,
    "start_hour": 361,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1242",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_16_187"
  },
  {
    "id": 1243,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1243",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_16_190"
  },
  {
    "id": 1244,
    "start_hour": 359,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1244",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_16_51,LEG_16_52"
  },
  {
    "id": 1245,
    "start_hour": 371,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1245",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_16_193"
  },
  {
    "id": 1246,
    "start_hour": 388,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1246",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_17_189"
  },
  {
    "id": 1247,
    "start_hour": 363,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1247",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_16_60,LEG_16_89,LEG_16_209,LEG_16_84"
  },
  {
    "id": 1248,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1248",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_16_90"
  },
  {
    "id": 1249,
    "start_hour": 392,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1249",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_17_88,LEG_17_81"
  },
  {
    "id": 1250,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1250",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_18_163,LEG_18_235,LEG_18_228"
  },
  {
    "id": 1251,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1251",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_16_119,LEG_16_114"
  },
  {
    "id": 1252,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1252",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_17_157,LEG_17_160,LEG_17_0"
  },
  {
    "id": 1253,
    "start_hour": 406,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1253",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_18_227,LEG_18_248,LEG_18_53"
  },
  {
    "id": 1254,
    "start_hour": 359,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1254",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_16_80"
  },
  {
    "id": 1255,
    "start_hour": 381,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1255",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_17_125,LEG_17_246"
  },
  {
    "id": 1256,
    "start_hour": 402,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1256",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_18_55,LEG_18_232,LEG_18_117"
  },
  {
    "id": 1257,
    "start_hour": 364,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1257",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_16_230,LEG_16_135"
  },
  {
    "id": 1258,
    "start_hour": 372,
    "duration_hours": 28,
    "required_skill": "A319",
    "gerad_duty_id": "D1258",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_17_87,LEG_17_30,LEG_17_21"
  },
  {
    "id": 1259,
    "start_hour": 412,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1259",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_18_64"
  },
  {
    "id": 1260,
    "start_hour": 371,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1260",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_16_37"
  },
  {
    "id": 1261,
    "start_hour": 386,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1261",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_17_5,LEG_17_142"
  },
  {
    "id": 1262,
    "start_hour": 413,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1262",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_18_208,LEG_18_127"
  },
  {
    "id": 1263,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1263",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_19_180,LEG_19_178,LEG_19_175"
  },
  {
    "id": 1264,
    "start_hour": 369,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1264",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_16_219,LEG_16_111"
  },
  {
    "id": 1265,
    "start_hour": 373,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1265",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_17_112,LEG_17_106,LEG_17_177"
  },
  {
    "id": 1266,
    "start_hour": 407,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1266",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_18_247,LEG_18_67"
  },
  {
    "id": 1267,
    "start_hour": 426,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1267",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_19_55"
  },
  {
    "id": 1268,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1268",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_16_65"
  },
  {
    "id": 1269,
    "start_hour": 386,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1269",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_17_16,LEG_17_162"
  },
  {
    "id": 1270,
    "start_hour": 407,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1270",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_18_14,LEG_18_27"
  },
  {
    "id": 1271,
    "start_hour": 426,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1271",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_19_35"
  },
  {
    "id": 1272,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1272",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_16_223"
  },
  {
    "id": 1273,
    "start_hour": 386,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1273",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_17_179,LEG_17_180,LEG_17_6"
  },
  {
    "id": 1274,
    "start_hour": 396,
    "duration_hours": 28,
    "required_skill": "A320",
    "gerad_duty_id": "D1274",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_18_87,LEG_18_30,LEG_18_21"
  },
  {
    "id": 1275,
    "start_hour": 436,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1275",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_19_56"
  },
  {
    "id": 1276,
    "start_hour": 365,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1276",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_16_86"
  },
  {
    "id": 1277,
    "start_hour": 393,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1277",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_17_57"
  },
  {
    "id": 1278,
    "start_hour": 398,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1278",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_18_121,LEG_18_79"
  },
  {
    "id": 1279,
    "start_hour": 104,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1279",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_05_47"
  },
  {
    "id": 1280,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1280",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_06_2,LEG_06_164"
  },
  {
    "id": 1281,
    "start_hour": 143,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1281",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_07_250,LEG_07_191"
  },
  {
    "id": 1282,
    "start_hour": 172,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1282",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_08_193"
  },
  {
    "id": 1283,
    "start_hour": 107,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1283",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_05_33"
  },
  {
    "id": 1284,
    "start_hour": 124,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1284",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_06_35"
  },
  {
    "id": 1285,
    "start_hour": 144,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1285",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_07_162,LEG_07_28"
  },
  {
    "id": 1286,
    "start_hour": 162,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1286",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_08_40"
  },
  {
    "id": 1287,
    "start_hour": 104,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1287",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_05_77"
  },
  {
    "id": 1288,
    "start_hour": 129,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1288",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_06_50"
  },
  {
    "id": 1289,
    "start_hour": 135,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1289",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_07_211,LEG_07_6,LEG_07_22"
  },
  {
    "id": 1290,
    "start_hour": 172,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1290",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_08_64"
  },
  {
    "id": 1291,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1291",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_05_204,LEG_05_81"
  },
  {
    "id": 1292,
    "start_hour": 109,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1292",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_06_126,LEG_06_22,LEG_06_10"
  },
  {
    "id": 1293,
    "start_hour": 140,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1293",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_07_10,LEG_07_71,LEG_07_140"
  },
  {
    "id": 1294,
    "start_hour": 160,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1294",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_08_2,LEG_08_62"
  },
  {
    "id": 1295,
    "start_hour": 107,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1295",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_05_178"
  },
  {
    "id": 1296,
    "start_hour": 124,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1296",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_06_177,LEG_06_36"
  },
  {
    "id": 1297,
    "start_hour": 143,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1297",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_07_217,LEG_07_66"
  },
  {
    "id": 1298,
    "start_hour": 162,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1298",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_08_63"
  },
  {
    "id": 1299,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1299",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_05_107,LEG_05_104"
  },
  {
    "id": 1300,
    "start_hour": 110,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1300",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_06_163,LEG_06_165"
  },
  {
    "id": 1301,
    "start_hour": 132,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1301",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_07_170,LEG_07_189"
  },
  {
    "id": 1302,
    "start_hour": 158,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1302",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_08_25,LEG_08_66"
  },
  {
    "id": 1303,
    "start_hour": 105,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1303",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_05_12,LEG_05_100"
  },
  {
    "id": 1304,
    "start_hour": 109,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1304",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_06_101,LEG_06_97,LEG_06_0"
  },
  {
    "id": 1305,
    "start_hour": 142,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1305",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_07_227,LEG_07_251,LEG_07_54"
  },
  {
    "id": 1306,
    "start_hour": 97,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1306",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_05_194,LEG_05_221,LEG_05_200"
  },
  {
    "id": 1307,
    "start_hour": 122,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1307",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_06_166,LEG_06_150"
  },
  {
    "id": 1308,
    "start_hour": 143,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1308",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_07_15,LEG_07_188"
  },
  {
    "id": 1309,
    "start_hour": 97,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1309",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_05_172"
  },
  {
    "id": 1310,
    "start_hour": 95,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1310",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_05_44,LEG_05_45"
  },
  {
    "id": 1311,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1311",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_10_223,LEG_10_24"
  },
  {
    "id": 1312,
    "start_hour": 230,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1312",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_11_106,LEG_11_163,LEG_11_234,LEG_11_138,LEG_11_179"
  },
  {
    "id": 1313,
    "start_hour": 264,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1313",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_12_159,LEG_12_27"
  },
  {
    "id": 1314,
    "start_hour": 282,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1314",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_13_33"
  },
  {
    "id": 1315,
    "start_hour": 224,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1315",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_10_89"
  },
  {
    "id": 1316,
    "start_hour": 249,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1316",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_11_58"
  },
  {
    "id": 1317,
    "start_hour": 255,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1317",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_12_189,LEG_12_5"
  },
  {
    "id": 1318,
    "start_hour": 276,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1318",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_13_136,LEG_13_192,LEG_13_187,LEG_13_184"
  },
  {
    "id": 1319,
    "start_hour": 224,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1319",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_10_55"
  },
  {
    "id": 1320,
    "start_hour": 246,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1320",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_11_3"
  },
  {
    "id": 1321,
    "start_hour": 252,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1321",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_12_79,LEG_12_209,LEG_12_99,LEG_12_103"
  },
  {
    "id": 1322,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1322",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_10_214,LEG_10_31,LEG_10_0"
  },
  {
    "id": 1323,
    "start_hour": 238,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1323",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_11_224,LEG_11_110"
  },
  {
    "id": 1324,
    "start_hour": 264,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1324",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_12_100,LEG_12_217,LEG_12_36"
  },
  {
    "id": 1325,
    "start_hour": 215,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1325",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_10_79"
  },
  {
    "id": 1326,
    "start_hour": 237,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1326",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_11_126,LEG_11_246"
  },
  {
    "id": 1327,
    "start_hour": 258,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1327",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_12_49,LEG_12_216,LEG_12_218"
  },
  {
    "id": 1328,
    "start_hour": 222,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1328",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_10_168,LEG_10_78,LEG_10_228"
  },
  {
    "id": 1329,
    "start_hour": 240,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1329",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_11_237,LEG_11_28"
  },
  {
    "id": 1330,
    "start_hour": 258,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1330",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_12_37"
  },
  {
    "id": 1331,
    "start_hour": 225,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1331",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_10_13,LEG_10_110"
  },
  {
    "id": 1332,
    "start_hour": 229,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1332",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_11_113,LEG_11_107,LEG_11_183"
  },
  {
    "id": 1333,
    "start_hour": 263,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1333",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_12_98,LEG_12_108,LEG_12_199"
  },
  {
    "id": 1334,
    "start_hour": 217,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1334",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_10_211,LEG_10_210,LEG_10_44"
  },
  {
    "id": 1335,
    "start_hour": 239,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1335",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_11_214,LEG_11_248,LEG_11_54"
  },
  {
    "id": 1336,
    "start_hour": 220,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1336",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_10_227,LEG_10_136"
  },
  {
    "id": 1337,
    "start_hour": 228,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1337",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_11_9,LEG_11_155,LEG_11_240,LEG_11_46"
  },
  {
    "id": 1338,
    "start_hour": 227,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1338",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_10_193"
  },
  {
    "id": 1339,
    "start_hour": 244,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1339",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_11_191"
  },
  {
    "id": 1340,
    "start_hour": 222,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1340",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_10_50,LEG_10_46,LEG_10_38"
  },
  {
    "id": 1341,
    "start_hour": 244,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1341",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_11_43"
  },
  {
    "id": 1342,
    "start_hour": 224,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1342",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_10_190"
  },
  {
    "id": 1343,
    "start_hour": 215,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1343",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_10_52,LEG_10_53"
  },
  {
    "id": 1344,
    "start_hour": 219,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1344",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_10_61,LEG_10_88,LEG_10_209,LEG_10_83"
  },
  {
    "id": 1345,
    "start_hour": 481,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1345",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_21_79,LEG_21_251"
  },
  {
    "id": 1346,
    "start_hour": 471,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1346",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_21_152,LEG_21_185"
  },
  {
    "id": 1347,
    "start_hour": 472,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1347",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_21_117,LEG_21_12"
  },
  {
    "id": 1348,
    "start_hour": 471,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1348",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_21_35,LEG_21_218"
  },
  {
    "id": 1349,
    "start_hour": 480,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1349",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_21_253,LEG_21_254"
  },
  {
    "id": 1350,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1350",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_21_15,LEG_21_22"
  },
  {
    "id": 1351,
    "start_hour": 483,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1351",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_21_98,LEG_21_101"
  },
  {
    "id": 1352,
    "start_hour": 484,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1352",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_21_78,LEG_21_122"
  },
  {
    "id": 1353,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1353",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_22_123,LEG_22_76"
  },
  {
    "id": 1354,
    "start_hour": 479,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1354",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_21_257,LEG_21_59"
  },
  {
    "id": 1355,
    "start_hour": 500,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1355",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_22_85,LEG_22_129,LEG_22_134"
  },
  {
    "id": 1356,
    "start_hour": 480,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1356",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_21_140,LEG_21_211,LEG_21_43"
  },
  {
    "id": 1357,
    "start_hour": 503,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1357",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_22_212,LEG_22_226,LEG_22_140"
  },
  {
    "id": 1358,
    "start_hour": 487,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1358",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_21_147,LEG_21_151"
  },
  {
    "id": 1359,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1359",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_21_157,LEG_21_104"
  },
  {
    "id": 1360,
    "start_hour": 487,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1360",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_21_115,LEG_21_118"
  },
  {
    "id": 1361,
    "start_hour": 488,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1361",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_21_252,LEG_21_1"
  },
  {
    "id": 1362,
    "start_hour": 484,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1362",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_21_94,LEG_21_93"
  },
  {
    "id": 1363,
    "start_hour": 485,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1363",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_21_18,LEG_21_25"
  },
  {
    "id": 1364,
    "start_hour": 470,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1364",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_21_245,LEG_21_242"
  },
  {
    "id": 1365,
    "start_hour": 492,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1365",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_22_146,LEG_22_200,LEG_22_201"
  },
  {
    "id": 1366,
    "start_hour": 530,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1366",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_23_72,LEG_23_180,LEG_23_6"
  },
  {
    "id": 1367,
    "start_hour": 468,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1367",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_21_75,LEG_21_128,LEG_21_239"
  },
  {
    "id": 1368,
    "start_hour": 493,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1368",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_22_13,LEG_22_172,LEG_22_239"
  },
  {
    "id": 1369,
    "start_hour": 530,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1369",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_23_242,LEG_23_97,LEG_23_213"
  },
  {
    "id": 1370,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1370",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_21_186,LEG_21_87"
  },
  {
    "id": 1371,
    "start_hour": 513,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1371",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_22_57"
  },
  {
    "id": 1372,
    "start_hour": 518,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1372",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_23_121,LEG_23_76"
  },
  {
    "id": 1373,
    "start_hour": 489,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1373",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_21_143"
  },
  {
    "id": 1374,
    "start_hour": 507,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1374",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_22_143,LEG_22_132"
  },
  {
    "id": 1375,
    "start_hour": 517,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1375",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_23_131"
  },
  {
    "id": 1376,
    "start_hour": 468,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1376",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_21_148,LEG_21_201,LEG_21_196,LEG_21_193,LEG_21_46"
  },
  {
    "id": 1377,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1377",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_22_163,LEG_22_4,LEG_22_215"
  },
  {
    "id": 1378,
    "start_hour": 491,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1378",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_21_31"
  },
  {
    "id": 1379,
    "start_hour": 515,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1379",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_22_36"
  },
  {
    "id": 1380,
    "start_hour": 485,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1380",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_21_154,LEG_21_149,LEG_21_233"
  },
  {
    "id": 1381,
    "start_hour": 504,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1381",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_22_238,LEG_22_173"
  },
  {
    "id": 1382,
    "start_hour": 528,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1382",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_23_174,LEG_23_11"
  },
  {
    "id": 1383,
    "start_hour": 548,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1383",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_24_10,LEG_24_71,LEG_24_137"
  },
  {
    "id": 1384,
    "start_hour": 481,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1384",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_21_171,LEG_21_222,LEG_21_63"
  },
  {
    "id": 1385,
    "start_hour": 502,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1385",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_22_225,LEG_22_109"
  },
  {
    "id": 1386,
    "start_hour": 528,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1386",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_23_110,LEG_23_41"
  },
  {
    "id": 1387,
    "start_hour": 554,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1387",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_24_32"
  },
  {
    "id": 1388,
    "start_hour": 485,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1388",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_21_166,LEG_21_167"
  },
  {
    "id": 1389,
    "start_hour": 492,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1389",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_22_9,LEG_22_153,LEG_22_142"
  },
  {
    "id": 1390,
    "start_hour": 533,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1390",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_23_208,LEG_23_127"
  },
  {
    "id": 1391,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1391",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_24_198,LEG_24_197,LEG_24_104"
  },
  {
    "id": 1392,
    "start_hour": 489,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1392",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_21_161,LEG_21_216"
  },
  {
    "id": 1393,
    "start_hour": 494,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1393",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_22_243,LEG_22_240"
  },
  {
    "id": 1394,
    "start_hour": 516,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1394",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_23_166,LEG_23_185"
  },
  {
    "id": 1395,
    "start_hour": 542,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1395",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_24_33,LEG_24_36"
  },
  {
    "id": 1396,
    "start_hour": 489,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1396",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_21_95,LEG_21_177"
  },
  {
    "id": 1397,
    "start_hour": 494,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1397",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_22_24,LEG_22_66"
  },
  {
    "id": 1398,
    "start_hour": 517,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D1398",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_23_13,LEG_23_172,LEG_23_239"
  },
  {
    "id": 1399,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1399",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_24_242,LEG_24_93,LEG_24_175"
  },
  {
    "id": 1400,
    "start_hour": 135,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1400",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_07_36,LEG_07_222"
  },
  {
    "id": 1401,
    "start_hour": 136,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1401",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_07_116,LEG_07_12"
  },
  {
    "id": 1402,
    "start_hour": 135,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1402",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_07_154,LEG_07_187"
  },
  {
    "id": 1403,
    "start_hour": 144,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1403",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_07_254,LEG_07_255"
  },
  {
    "id": 1404,
    "start_hour": 147,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1404",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_07_97,LEG_07_100"
  },
  {
    "id": 1405,
    "start_hour": 145,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1405",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_07_173,LEG_07_174"
  },
  {
    "id": 1406,
    "start_hour": 132,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1406",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_07_74,LEG_07_70"
  },
  {
    "id": 1407,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1407",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_07_16,LEG_07_23"
  },
  {
    "id": 1408,
    "start_hour": 145,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1408",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_07_78,LEG_07_252"
  },
  {
    "id": 1409,
    "start_hour": 144,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1409",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_07_141,LEG_07_215,LEG_07_44"
  },
  {
    "id": 1410,
    "start_hour": 167,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1410",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_08_216,LEG_08_225,LEG_08_144"
  },
  {
    "id": 1411,
    "start_hour": 148,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1411",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_07_77,LEG_07_121"
  },
  {
    "id": 1412,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1412",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_08_124,LEG_08_76"
  },
  {
    "id": 1413,
    "start_hour": 143,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1413",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_07_258,LEG_07_60"
  },
  {
    "id": 1414,
    "start_hour": 164,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1414",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_08_86,LEG_08_130,LEG_08_137"
  },
  {
    "id": 1415,
    "start_hour": 134,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1415",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_07_34,LEG_07_30,LEG_07_101"
  },
  {
    "id": 1416,
    "start_hour": 167,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1416",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_08_217,LEG_08_215,LEG_08_143"
  },
  {
    "id": 1417,
    "start_hour": 151,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1417",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_07_149,LEG_07_153"
  },
  {
    "id": 1418,
    "start_hour": 151,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1418",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_07_114,LEG_07_117"
  },
  {
    "id": 1419,
    "start_hour": 152,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1419",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_07_253,LEG_07_1"
  },
  {
    "id": 1420,
    "start_hour": 149,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1420",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_07_19,LEG_07_26"
  },
  {
    "id": 1421,
    "start_hour": 148,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1421",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_07_93,LEG_07_92"
  },
  {
    "id": 1422,
    "start_hour": 136,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1422",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_07_2,LEG_07_62"
  },
  {
    "id": 1423,
    "start_hour": 157,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1423",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_08_195,LEG_08_125"
  },
  {
    "id": 1424,
    "start_hour": 182,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1424",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_09_59,LEG_09_96"
  },
  {
    "id": 1425,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1425",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_07_159,LEG_07_103"
  },
  {
    "id": 1426,
    "start_hour": 159,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1426",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_08_136,LEG_08_228,LEG_08_65"
  },
  {
    "id": 1427,
    "start_hour": 194,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1427",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_09_17"
  },
  {
    "id": 1428,
    "start_hour": 155,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1428",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_07_32"
  },
  {
    "id": 1429,
    "start_hour": 179,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1429",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_08_37"
  },
  {
    "id": 1430,
    "start_hour": 134,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1430",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_07_25,LEG_07_65"
  },
  {
    "id": 1431,
    "start_hour": 157,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1431",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_08_49,LEG_08_51,LEG_08_50,LEG_08_35"
  },
  {
    "id": 1432,
    "start_hour": 134,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1432",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_07_245,LEG_07_242"
  },
  {
    "id": 1433,
    "start_hour": 156,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1433",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_08_170,LEG_08_189"
  },
  {
    "id": 1434,
    "start_hour": 182,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1434",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_09_102,LEG_09_95,LEG_09_133"
  },
  {
    "id": 1435,
    "start_hour": 205,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1435",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_10_130"
  },
  {
    "id": 1436,
    "start_hour": 145,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1436",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_07_186,LEG_07_84,LEG_07_192"
  },
  {
    "id": 1437,
    "start_hour": 169,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1437",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_08_41,LEG_08_42"
  },
  {
    "id": 1438,
    "start_hour": 194,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1438",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_09_33,LEG_09_155"
  },
  {
    "id": 1439,
    "start_hour": 216,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1439",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_10_158,LEG_10_129,LEG_10_132"
  },
  {
    "id": 1440,
    "start_hour": 149,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1440",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_07_156,LEG_07_151,LEG_07_233"
  },
  {
    "id": 1441,
    "start_hour": 168,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1441",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_08_239,LEG_08_177"
  },
  {
    "id": 1442,
    "start_hour": 192,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1442",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_09_178,LEG_09_11"
  },
  {
    "id": 1443,
    "start_hour": 212,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1443",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_10_10,LEG_10_70,LEG_10_138"
  },
  {
    "id": 1444,
    "start_hour": 148,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1444",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_07_148,LEG_07_208,LEG_07_128"
  },
  {
    "id": 1445,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1445",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_08_202,LEG_08_199,LEG_08_196,LEG_08_47"
  },
  {
    "id": 1446,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1446",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_09_167,LEG_09_4,LEG_09_135"
  },
  {
    "id": 1447,
    "start_hour": 153,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1447",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_07_94,LEG_07_179"
  },
  {
    "id": 1448,
    "start_hour": 158,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1448",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_08_102,LEG_08_95,LEG_08_255"
  },
  {
    "id": 1449,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1449",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_09_158,LEG_09_73,LEG_09_160"
  },
  {
    "id": 1450,
    "start_hour": 149,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1450",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_07_168,LEG_07_169"
  },
  {
    "id": 1451,
    "start_hour": 156,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D1451",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_08_29,LEG_08_69,LEG_08_68"
  },
  {
    "id": 1452,
    "start_hour": 196,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1452",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_09_21,LEG_09_255"
  },
  {
    "id": 1453,
    "start_hour": 218,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1453",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_10_154,LEG_10_4,LEG_10_133"
  },
  {
    "id": 1454,
    "start_hour": 153,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1454",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_07_163,LEG_07_221"
  },
  {
    "id": 1455,
    "start_hour": 158,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1455",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_08_161,LEG_08_164"
  },
  {
    "id": 1456,
    "start_hour": 180,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1456",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_09_170,LEG_09_189"
  },
  {
    "id": 1457,
    "start_hour": 206,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1457",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_10_104,LEG_10_161,LEG_10_72,LEG_10_156"
  },
  {
    "id": 1458,
    "start_hour": 145,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1458",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_07_45,LEG_07_171,LEG_07_224"
  },
  {
    "id": 1459,
    "start_hour": 170,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1459",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_08_183,LEG_08_184,LEG_08_6"
  },
  {
    "id": 1460,
    "start_hour": 180,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1460",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_09_88,LEG_09_246,LEG_09_172,LEG_09_80"
  },
  {
    "id": 1461,
    "start_hour": 204,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1461",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_10_86,LEG_10_242,LEG_10_118,LEG_10_113"
  },
  {
    "id": 1462,
    "start_hour": 481,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1462",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_21_189"
  },
  {
    "id": 1463,
    "start_hour": 488,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1463",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_21_192"
  },
  {
    "id": 1464,
    "start_hour": 469,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1464",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_21_47,LEG_21_188"
  },
  {
    "id": 1465,
    "start_hour": 479,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1465",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_21_51,LEG_21_52"
  },
  {
    "id": 1466,
    "start_hour": 481,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1466",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_21_67"
  },
  {
    "id": 1467,
    "start_hour": 498,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1467",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_22_62"
  },
  {
    "id": 1468,
    "start_hour": 470,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1468",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_21_225,LEG_21_14"
  },
  {
    "id": 1469,
    "start_hour": 496,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1469",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_22_2,LEG_22_61"
  },
  {
    "id": 1470,
    "start_hour": 491,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1470",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_21_194"
  },
  {
    "id": 1471,
    "start_hour": 508,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1471",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_22_189"
  },
  {
    "id": 1472,
    "start_hour": 482,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1472",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_21_120,LEG_21_227,LEG_21_210,LEG_21_85"
  },
  {
    "id": 1473,
    "start_hour": 492,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1473",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_21_68"
  },
  {
    "id": 1474,
    "start_hour": 508,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1474",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_22_20,LEG_22_107"
  },
  {
    "id": 1475,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1475",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_23_98,LEG_23_241,LEG_23_45"
  },
  {
    "id": 1476,
    "start_hour": 479,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1476",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_21_81"
  },
  {
    "id": 1477,
    "start_hour": 501,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1477",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_22_125,LEG_22_246"
  },
  {
    "id": 1478,
    "start_hour": 522,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1478",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_23_55,LEG_23_232,LEG_23_117"
  },
  {
    "id": 1479,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1479",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_21_121,LEG_21_116"
  },
  {
    "id": 1480,
    "start_hour": 494,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1480",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_22_157,LEG_22_160,LEG_22_7"
  },
  {
    "id": 1481,
    "start_hour": 532,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1481",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_23_42"
  },
  {
    "id": 1482,
    "start_hour": 486,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1482",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_21_49,LEG_21_34"
  },
  {
    "id": 1483,
    "start_hour": 494,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1483",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_22_176,LEG_22_178"
  },
  {
    "id": 1484,
    "start_hour": 516,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1484",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_23_9,LEG_23_153,LEG_23_77"
  },
  {
    "id": 1485,
    "start_hour": 548,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1485",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_24_70,LEG_24_182,LEG_24_83"
  },
  {
    "id": 1486,
    "start_hour": 489,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1486",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_21_221,LEG_21_113"
  },
  {
    "id": 1487,
    "start_hour": 493,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1487",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_22_112,LEG_22_106"
  },
  {
    "id": 1488,
    "start_hour": 516,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1488",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_23_146,LEG_23_200,LEG_23_201"
  },
  {
    "id": 1489,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1489",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_24_122,LEG_24_195,LEG_24_192"
  },
  {
    "id": 1490,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1490",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_21_60,LEG_21_90,LEG_21_91"
  },
  {
    "id": 1491,
    "start_hour": 512,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1491",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_22_88,LEG_22_100"
  },
  {
    "id": 1492,
    "start_hour": 528,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1492",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_23_26,LEG_23_27"
  },
  {
    "id": 1493,
    "start_hour": 546,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1493",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_24_39"
  },
  {
    "id": 1494,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1494",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_21_169,LEG_21_23"
  },
  {
    "id": 1495,
    "start_hour": 494,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1495",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_22_105,LEG_22_161,LEG_22_97,LEG_22_213,LEG_22_81"
  },
  {
    "id": 1496,
    "start_hour": 531,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1496",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_23_194,LEG_23_195,LEG_23_192,LEG_23_46"
  },
  {
    "id": 1497,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1497",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_24_163,LEG_24_235,LEG_24_228"
  },
  {
    "id": 1498,
    "start_hour": 484,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1498",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_21_232,LEG_21_137"
  },
  {
    "id": 1499,
    "start_hour": 492,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1499",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_22_166,LEG_22_185"
  },
  {
    "id": 1500,
    "start_hour": 518,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1500",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_23_24,LEG_23_66"
  },
  {
    "id": 1501,
    "start_hour": 432,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1501",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_19_224,LEG_19_225"
  },
  {
    "id": 1502,
    "start_hour": 435,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1502",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_19_87,LEG_19_90"
  },
  {
    "id": 1503,
    "start_hour": 433,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1503",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_19_157,LEG_19_158"
  },
  {
    "id": 1504,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1504",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_19_15,LEG_19_21"
  },
  {
    "id": 1505,
    "start_hour": 433,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1505",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_19_67,LEG_19_222"
  },
  {
    "id": 1506,
    "start_hour": 423,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1506",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_19_32,LEG_19_195"
  },
  {
    "id": 1507,
    "start_hour": 424,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1507",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_19_108,LEG_19_12"
  },
  {
    "id": 1508,
    "start_hour": 423,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1508",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_19_140,LEG_19_167"
  },
  {
    "id": 1509,
    "start_hour": 432,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1509",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_19_128,LEG_19_129"
  },
  {
    "id": 1510,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1510",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_19_191,LEG_19_192"
  },
  {
    "id": 1511,
    "start_hour": 420,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1511",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_19_65,LEG_19_117,LEG_19_229"
  },
  {
    "id": 1512,
    "start_hour": 445,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1512",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_20_170"
  },
  {
    "id": 1513,
    "start_hour": 443,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1513",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_19_29"
  },
  {
    "id": 1514,
    "start_hour": 464,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1514",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_20_24"
  },
  {
    "id": 1515,
    "start_hour": 439,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1515",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_19_135,LEG_19_139"
  },
  {
    "id": 1516,
    "start_hour": 437,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1516",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_19_17,LEG_19_23"
  },
  {
    "id": 1517,
    "start_hour": 437,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1517",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_19_82,LEG_19_81"
  },
  {
    "id": 1518,
    "start_hour": 436,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1518",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_19_79,LEG_19_78"
  },
  {
    "id": 1519,
    "start_hour": 440,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1519",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_19_223,LEG_19_1"
  },
  {
    "id": 1520,
    "start_hour": 439,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1520",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_19_213,LEG_19_39,LEG_19_33"
  },
  {
    "id": 1521,
    "start_hour": 460,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1521",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_20_34"
  },
  {
    "id": 1522,
    "start_hour": 480,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1522",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_21_160,LEG_21_132,LEG_21_135"
  },
  {
    "id": 1523,
    "start_hour": 422,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1523",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_19_208"
  },
  {
    "id": 1524,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1524",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_20_214,LEG_20_92,LEG_20_196,LEG_20_163"
  },
  {
    "id": 1525,
    "start_hour": 479,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1525",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_21_249,LEG_21_212,LEG_21_141"
  },
  {
    "id": 1526,
    "start_hour": 422,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1526",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_19_31,LEG_19_27,LEG_19_69"
  },
  {
    "id": 1527,
    "start_hour": 457,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1527",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_20_151,LEG_20_200,LEG_20_201"
  },
  {
    "id": 1528,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1528",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_19_4,LEG_19_194"
  },
  {
    "id": 1529,
    "start_hour": 446,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1529",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_20_100,LEG_20_150,LEG_20_108"
  },
  {
    "id": 1530,
    "start_hour": 437,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1530",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_19_153,LEG_19_154"
  },
  {
    "id": 1531,
    "start_hour": 444,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D1531",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_20_195,LEG_20_51,LEG_20_103"
  },
  {
    "id": 1532,
    "start_hour": 480,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1532",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_21_112,LEG_21_41"
  },
  {
    "id": 1533,
    "start_hour": 506,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1533",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_22_32"
  },
  {
    "id": 1534,
    "start_hour": 441,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1534",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_19_149,LEG_19_206"
  },
  {
    "id": 1535,
    "start_hour": 445,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1535",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_20_202,LEG_20_203,LEG_20_54"
  },
  {
    "id": 1536,
    "start_hour": 482,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1536",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_21_16,LEG_21_164"
  },
  {
    "id": 1537,
    "start_hour": 503,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1537",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_22_14,LEG_22_130,LEG_22_133"
  },
  {
    "id": 1538,
    "start_hour": 422,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1538",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_19_22,LEG_19_58"
  },
  {
    "id": 1539,
    "start_hour": 445,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1539",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_20_38,LEG_20_179,LEG_20_180,LEG_20_177,LEG_20_37"
  },
  {
    "id": 1540,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1540",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_21_165,LEG_21_4,LEG_21_217"
  },
  {
    "id": 1541,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1541",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_19_228,LEG_19_52"
  },
  {
    "id": 1542,
    "start_hour": 452,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1542",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_20_76,LEG_20_222"
  },
  {
    "id": 1543,
    "start_hour": 482,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1543",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_21_244,LEG_21_255"
  },
  {
    "id": 1544,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1544",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_22_154,LEG_22_73,LEG_22_156"
  },
  {
    "id": 1545,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1545",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_19_146,LEG_19_92"
  },
  {
    "id": 1546,
    "start_hour": 447,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1546",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_20_123,LEG_20_122,LEG_20_131"
  },
  {
    "id": 1547,
    "start_hour": 485,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1547",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_21_209,LEG_21_129"
  },
  {
    "id": 1548,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1548",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_22_198,LEG_22_197,LEG_22_104"
  },
  {
    "id": 1549,
    "start_hour": 314,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1549",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_14_257,LEG_14_260"
  },
  {
    "id": 1550,
    "start_hour": 314,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1550",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_14_206,LEG_14_205"
  },
  {
    "id": 1551,
    "start_hour": 303,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1551",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_14_207,LEG_14_102"
  },
  {
    "id": 1552,
    "start_hour": 325,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1552",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_15_191,LEG_15_124"
  },
  {
    "id": 1553,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1553",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_14_123,LEG_14_120"
  },
  {
    "id": 1554,
    "start_hour": 323,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1554",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_14_79"
  },
  {
    "id": 1555,
    "start_hour": 341,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1555",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_15_56,LEG_15_245"
  },
  {
    "id": 1556,
    "start_hour": 322,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1556",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_14_259"
  },
  {
    "id": 1557,
    "start_hour": 325,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1557",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_15_217,LEG_15_218,LEG_15_90"
  },
  {
    "id": 1558,
    "start_hour": 368,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1558",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_16_88,LEG_16_100"
  },
  {
    "id": 1559,
    "start_hour": 384,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1559",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_17_26,LEG_17_144,LEG_17_203"
  },
  {
    "id": 1560,
    "start_hour": 322,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1560",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_14_238"
  },
  {
    "id": 1561,
    "start_hour": 325,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1561",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_15_112,LEG_15_104"
  },
  {
    "id": 1562,
    "start_hour": 350,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1562",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_16_101,LEG_16_94,LEG_16_171,LEG_16_199"
  },
  {
    "id": 1563,
    "start_hour": 323,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1563",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_14_128"
  },
  {
    "id": 1564,
    "start_hour": 347,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1564",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_15_79"
  },
  {
    "id": 1565,
    "start_hour": 365,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1565",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_16_56,LEG_16_245"
  },
  {
    "id": 1566,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1566",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_14_203,LEG_14_149"
  },
  {
    "id": 1567,
    "start_hour": 326,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1567",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_15_176,LEG_15_178"
  },
  {
    "id": 1568,
    "start_hour": 348,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1568",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_16_166,LEG_16_185"
  },
  {
    "id": 1569,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1569",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_17_101,LEG_17_94,LEG_17_171,LEG_17_199"
  },
  {
    "id": 1570,
    "start_hour": 302,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1570",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_14_121,LEG_14_76"
  },
  {
    "id": 1571,
    "start_hour": 324,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1571",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_15_166,LEG_15_185"
  },
  {
    "id": 1572,
    "start_hour": 350,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1572",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_16_234"
  },
  {
    "id": 1573,
    "start_hour": 386,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1573",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_17_233,LEG_17_19,LEG_17_196"
  },
  {
    "id": 1574,
    "start_hour": 709,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1574",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_31_73,LEG_31_262"
  },
  {
    "id": 1575,
    "start_hour": 725,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1575",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_31_166,LEG_31_164"
  },
  {
    "id": 1576,
    "start_hour": 721,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1576",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_31_132,LEG_31_128"
  },
  {
    "id": 1577,
    "start_hour": 724,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1577",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_31_83,LEG_31_88"
  },
  {
    "id": 1578,
    "start_hour": 722,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1578",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_31_248,LEG_31_2"
  },
  {
    "id": 1579,
    "start_hour": 722,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1579",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_31_140,LEG_31_141"
  },
  {
    "id": 1580,
    "start_hour": 726,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1580",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_31_154,LEG_31_156"
  },
  {
    "id": 1581,
    "start_hour": 721,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1581",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_31_113,LEG_31_74"
  },
  {
    "id": 1582,
    "start_hour": 724,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1582",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_31_25,LEG_31_18"
  },
  {
    "id": 1583,
    "start_hour": 720,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1583",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_31_256,LEG_31_9"
  },
  {
    "id": 1584,
    "start_hour": 712,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1584",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_31_155,LEG_31_133"
  },
  {
    "id": 1585,
    "start_hour": 711,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1585",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_31_26,LEG_31_20"
  },
  {
    "id": 1586,
    "start_hour": 723,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1586",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_31_8,LEG_31_6"
  },
  {
    "id": 1587,
    "start_hour": 723,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1587",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_31_31,LEG_31_101"
  },
  {
    "id": 1588,
    "start_hour": 711,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1588",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_31_27,LEG_31_29"
  },
  {
    "id": 1589,
    "start_hour": 721,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1589",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_31_263,LEG_31_264"
  },
  {
    "id": 1590,
    "start_hour": 726,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1590",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_31_22,LEG_31_23"
  },
  {
    "id": 1591,
    "start_hour": 728,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1591",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_31_57,LEG_31_112"
  },
  {
    "id": 1592,
    "start_hour": 713,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1592",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_31_111,LEG_31_168"
  },
  {
    "id": 1593,
    "start_hour": 721,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1593",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_31_267,LEG_31_181"
  },
  {
    "id": 1594,
    "start_hour": 711,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D1594",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_31_146,LEG_31_81"
  },
  {
    "id": 1595,
    "start_hour": 328,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1595",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_15_115,LEG_15_12"
  },
  {
    "id": 1596,
    "start_hour": 339,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1596",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_15_96,LEG_15_99"
  },
  {
    "id": 1597,
    "start_hour": 336,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1597",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_15_251,LEG_15_252"
  },
  {
    "id": 1598,
    "start_hour": 337,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1598",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_15_78,LEG_15_249"
  },
  {
    "id": 1599,
    "start_hour": 327,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1599",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_15_35,LEG_15_216"
  },
  {
    "id": 1600,
    "start_hour": 327,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1600",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_15_150,LEG_15_183"
  },
  {
    "id": 1601,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1601",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_15_15,LEG_15_22"
  },
  {
    "id": 1602,
    "start_hour": 345,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1602",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_15_57"
  },
  {
    "id": 1603,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1603",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_16_207,LEG_16_8"
  },
  {
    "id": 1604,
    "start_hour": 336,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1604",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_15_138,LEG_15_210,LEG_15_43"
  },
  {
    "id": 1605,
    "start_hour": 359,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1605",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_16_212,LEG_16_221,LEG_16_170"
  },
  {
    "id": 1606,
    "start_hour": 335,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1606",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_15_255,LEG_15_59"
  },
  {
    "id": 1607,
    "start_hour": 356,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1607",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_16_85,LEG_16_129,LEG_16_134"
  },
  {
    "id": 1608,
    "start_hour": 344,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1608",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_15_250,LEG_15_1"
  },
  {
    "id": 1609,
    "start_hour": 343,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1609",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_15_145,LEG_15_149"
  },
  {
    "id": 1610,
    "start_hour": 340,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1610",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_15_92,LEG_15_91"
  },
  {
    "id": 1611,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1611",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_15_155,LEG_15_102"
  },
  {
    "id": 1612,
    "start_hour": 343,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1612",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_15_113,LEG_15_116"
  },
  {
    "id": 1613,
    "start_hour": 341,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1613",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_15_18,LEG_15_25"
  },
  {
    "id": 1614,
    "start_hour": 347,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1614",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_15_31"
  },
  {
    "id": 1615,
    "start_hour": 368,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1615",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_16_29,LEG_16_75"
  },
  {
    "id": 1616,
    "start_hour": 385,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1616",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_17_17,LEG_17_136,LEG_17_236"
  },
  {
    "id": 1617,
    "start_hour": 324,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1617",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_15_74,LEG_15_126,LEG_15_237"
  },
  {
    "id": 1618,
    "start_hour": 349,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1618",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_16_47,LEG_16_163,LEG_16_4,LEG_16_215"
  },
  {
    "id": 1619,
    "start_hour": 344,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1619",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_15_19,LEG_15_196"
  },
  {
    "id": 1620,
    "start_hour": 351,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1620",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_16_206,LEG_16_6,LEG_16_107"
  },
  {
    "id": 1621,
    "start_hour": 386,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1621",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_17_98,LEG_17_97,LEG_17_213,LEG_17_100"
  },
  {
    "id": 1622,
    "start_hour": 408,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1622",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_18_26,LEG_18_130,LEG_18_133"
  },
  {
    "id": 1623,
    "start_hour": 348,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1623",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_15_199"
  },
  {
    "id": 1624,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1624",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_16_121,LEG_16_79"
  },
  {
    "id": 1625,
    "start_hour": 389,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1625",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_17_56,LEG_17_245"
  },
  {
    "id": 1626,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1626",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_18_207,LEG_18_8"
  },
  {
    "id": 1627,
    "start_hour": 341,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1627",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_15_164,LEG_15_165"
  },
  {
    "id": 1628,
    "start_hour": 348,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D1628",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_16_28,LEG_16_69,LEG_16_68"
  },
  {
    "id": 1629,
    "start_hour": 388,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1629",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_17_20,LEG_17_132"
  },
  {
    "id": 1630,
    "start_hour": 397,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1630",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_18_131"
  },
  {
    "id": 1631,
    "start_hour": 336,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1631",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_15_71,LEG_15_137,LEG_15_162"
  },
  {
    "id": 1632,
    "start_hour": 359,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1632",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_16_14,LEG_16_173"
  },
  {
    "id": 1633,
    "start_hour": 384,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1633",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_17_174,LEG_17_11"
  },
  {
    "id": 1634,
    "start_hour": 404,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1634",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_18_10,LEG_18_71,LEG_18_137"
  },
  {
    "id": 1635,
    "start_hour": 337,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1635",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_15_169,LEG_15_220,LEG_15_63"
  },
  {
    "id": 1636,
    "start_hour": 358,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1636",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_16_225,LEG_16_109"
  },
  {
    "id": 1637,
    "start_hour": 384,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1637",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_17_110,LEG_17_41"
  },
  {
    "id": 1638,
    "start_hour": 410,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1638",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_18_32"
  },
  {
    "id": 1639,
    "start_hour": 345,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1639",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_15_159,LEG_15_214"
  },
  {
    "id": 1640,
    "start_hour": 350,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1640",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_16_24,LEG_16_66"
  },
  {
    "id": 1641,
    "start_hour": 373,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1641",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_17_48,LEG_17_50,LEG_17_65"
  },
  {
    "id": 1642,
    "start_hour": 410,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1642",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_18_16"
  },
  {
    "id": 1643,
    "start_hour": 326,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1643",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_15_243,LEG_15_240"
  },
  {
    "id": 1644,
    "start_hour": 348,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1644",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_16_146,LEG_16_200,LEG_16_201"
  },
  {
    "id": 1645,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1645",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_17_72,LEG_17_241,LEG_17_34"
  },
  {
    "id": 1646,
    "start_hour": 341,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1646",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_15_152,LEG_15_147,LEG_15_100"
  },
  {
    "id": 1647,
    "start_hour": 360,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1647",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_16_26,LEG_16_239"
  },
  {
    "id": 1648,
    "start_hour": 386,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1648",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_17_242,LEG_17_253"
  },
  {
    "id": 1649,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1649",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_18_154,LEG_18_73,LEG_18_156"
  },
  {
    "id": 1650,
    "start_hour": 347,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1650",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_15_7"
  },
  {
    "id": 1651,
    "start_hour": 362,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1651",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_16_5,LEG_16_142"
  },
  {
    "id": 1652,
    "start_hour": 389,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1652",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_17_208,LEG_17_127"
  },
  {
    "id": 1653,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1653",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_18_198,LEG_18_197,LEG_18_104"
  },
  {
    "id": 1654,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1654",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_15_44,LEG_15_168,LEG_15_54"
  },
  {
    "id": 1655,
    "start_hour": 366,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1655",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_16_3,LEG_16_7"
  },
  {
    "id": 1656,
    "start_hour": 388,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1656",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_17_42,LEG_17_188"
  },
  {
    "id": 1657,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1657",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_18_40,LEG_18_119,LEG_18_114"
  },
  {
    "id": 1658,
    "start_hour": 470,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1658",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_21_58,LEG_21_97"
  },
  {
    "id": 1659,
    "start_hour": 494,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1659",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_22_234"
  },
  {
    "id": 1660,
    "start_hour": 530,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1660",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_23_233,LEG_23_151"
  },
  {
    "id": 1661,
    "start_hour": 552,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1661",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_24_158,LEG_24_144,LEG_24_203"
  },
  {
    "id": 1662,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1662",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_21_208,LEG_21_8"
  },
  {
    "id": 1663,
    "start_hour": 505,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1663",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_22_182,LEG_22_83,LEG_22_188"
  },
  {
    "id": 1664,
    "start_hour": 529,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1664",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_23_40,LEG_23_86"
  },
  {
    "id": 1665,
    "start_hour": 561,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1665",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_24_57"
  },
  {
    "id": 1666,
    "start_hour": 490,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1666",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_21_258"
  },
  {
    "id": 1667,
    "start_hour": 493,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1667",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_22_217,LEG_22_218,LEG_22_82"
  },
  {
    "id": 1668,
    "start_hour": 530,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1668",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_23_122"
  },
  {
    "id": 1669,
    "start_hour": 491,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1669",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_21_130"
  },
  {
    "id": 1670,
    "start_hour": 510,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1670",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_22_126,LEG_22_128"
  },
  {
    "id": 1671,
    "start_hour": 534,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1671",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_23_126"
  },
  {
    "id": 1672,
    "start_hour": 471,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1672",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_21_207,LEG_21_105"
  },
  {
    "id": 1673,
    "start_hour": 493,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1673",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_22_191,LEG_22_124"
  },
  {
    "id": 1674,
    "start_hour": 470,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1674",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_21_123,LEG_21_127"
  },
  {
    "id": 1675,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1675",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_21_206,LEG_21_205"
  },
  {
    "id": 1676,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1676",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_21_256,LEG_21_259"
  },
  {
    "id": 1677,
    "start_hour": 49,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1677",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_03_191"
  },
  {
    "id": 1678,
    "start_hour": 52,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1678",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_03_231,LEG_03_138"
  },
  {
    "id": 1679,
    "start_hour": 60,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1679",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_04_88,LEG_04_172,LEG_04_80"
  },
  {
    "id": 1680,
    "start_hour": 84,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1680",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_05_8,LEG_05_144,LEG_05_152"
  },
  {
    "id": 1681,
    "start_hour": 119,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1681",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_06_12,LEG_06_170"
  },
  {
    "id": 1682,
    "start_hour": 56,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1682",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_03_91"
  },
  {
    "id": 1683,
    "start_hour": 80,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1683",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_04_89,LEG_04_101"
  },
  {
    "id": 1684,
    "start_hour": 95,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1684",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_05_193,LEG_05_59"
  },
  {
    "id": 1685,
    "start_hour": 114,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1685",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_06_55"
  },
  {
    "id": 1686,
    "start_hour": 51,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1686",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_03_61,LEG_03_90,LEG_03_83"
  },
  {
    "id": 1687,
    "start_hour": 74,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1687",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_04_123,LEG_04_205"
  },
  {
    "id": 1688,
    "start_hour": 93,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1688",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_05_113,LEG_05_224"
  },
  {
    "id": 1689,
    "start_hour": 114,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1689",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_06_48,LEG_06_198,LEG_06_229"
  },
  {
    "id": 1690,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1690",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_03_227,LEG_03_24"
  },
  {
    "id": 1691,
    "start_hour": 62,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1691",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_04_106,LEG_04_165,LEG_04_166"
  },
  {
    "id": 1692,
    "start_hour": 95,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1692",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_05_14,LEG_05_25"
  },
  {
    "id": 1693,
    "start_hour": 114,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1693",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_06_32,LEG_06_205,LEG_06_212"
  },
  {
    "id": 1694,
    "start_hour": 54,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1694",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_03_245,LEG_03_39"
  },
  {
    "id": 1695,
    "start_hour": 61,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D1695",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_04_14,LEG_04_176,LEG_04_240"
  },
  {
    "id": 1696,
    "start_hour": 98,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1696",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_05_217,LEG_05_166"
  },
  {
    "id": 1697,
    "start_hour": 119,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1697",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_06_94,LEG_06_232,LEG_06_46"
  },
  {
    "id": 1698,
    "start_hour": 59,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1698",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_03_197"
  },
  {
    "id": 1699,
    "start_hour": 76,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1699",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_04_193,LEG_04_13,LEG_04_112"
  },
  {
    "id": 1700,
    "start_hour": 85,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1700",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_05_13,LEG_05_170,LEG_05_125,LEG_05_207"
  },
  {
    "id": 1701,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1701",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_03_218,LEG_03_31,LEG_03_0"
  },
  {
    "id": 1702,
    "start_hour": 70,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1702",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_04_226,LEG_04_110"
  },
  {
    "id": 1703,
    "start_hour": 96,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1703",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_05_99,LEG_05_220,LEG_05_34"
  },
  {
    "id": 1704,
    "start_hour": 47,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1704",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_03_81"
  },
  {
    "id": 1705,
    "start_hour": 69,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1705",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_04_126,LEG_04_248"
  },
  {
    "id": 1706,
    "start_hour": 90,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1706",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_05_48,LEG_05_219,LEG_05_222"
  },
  {
    "id": 1707,
    "start_hour": 56,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1707",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_03_55"
  },
  {
    "id": 1708,
    "start_hour": 78,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1708",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_04_3,LEG_04_22"
  },
  {
    "id": 1709,
    "start_hour": 100,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1709",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_05_56"
  },
  {
    "id": 1710,
    "start_hour": 57,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1710",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_03_13,LEG_03_112"
  },
  {
    "id": 1711,
    "start_hour": 61,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1711",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_04_113,LEG_04_107,LEG_04_185"
  },
  {
    "id": 1712,
    "start_hour": 95,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1712",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_05_97,LEG_05_53,LEG_05_76"
  },
  {
    "id": 1713,
    "start_hour": 54,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1713",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_03_172,LEG_03_80"
  },
  {
    "id": 1714,
    "start_hour": 60,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1714",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_04_9,LEG_04_157,LEG_04_98,LEG_04_219,LEG_04_175"
  },
  {
    "id": 1715,
    "start_hour": 95,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1715",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_05_91,LEG_05_225,LEG_05_46"
  },
  {
    "id": 1716,
    "start_hour": 59,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1716",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_03_38"
  },
  {
    "id": 1717,
    "start_hour": 76,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1717",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_04_43"
  },
  {
    "id": 1718,
    "start_hour": 56,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1718",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_03_194"
  },
  {
    "id": 1719,
    "start_hour": 37,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1719",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_03_48,LEG_03_190"
  },
  {
    "id": 1720,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1720",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_03_52,LEG_03_53"
  },
  {
    "id": 1721,
    "start_hour": 351,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1721",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_16_35,LEG_16_216"
  },
  {
    "id": 1722,
    "start_hour": 352,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1722",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_16_115,LEG_16_12"
  },
  {
    "id": 1723,
    "start_hour": 351,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1723",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_16_150,LEG_16_183"
  },
  {
    "id": 1724,
    "start_hour": 360,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1724",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_16_251,LEG_16_252"
  },
  {
    "id": 1725,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1725",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_16_15,LEG_16_22"
  },
  {
    "id": 1726,
    "start_hour": 363,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1726",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_16_96,LEG_16_99"
  },
  {
    "id": 1727,
    "start_hour": 361,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1727",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_16_78,LEG_16_249"
  },
  {
    "id": 1728,
    "start_hour": 362,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1728",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_16_184,LEG_16_41"
  },
  {
    "id": 1729,
    "start_hour": 386,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1729",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_17_32"
  },
  {
    "id": 1730,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1730",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_16_44,LEG_16_168,LEG_16_54"
  },
  {
    "id": 1731,
    "start_hour": 390,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1731",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_17_3"
  },
  {
    "id": 1732,
    "start_hour": 359,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1732",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_16_255,LEG_16_59"
  },
  {
    "id": 1733,
    "start_hour": 380,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1733",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_17_85,LEG_17_129,LEG_17_134"
  },
  {
    "id": 1734,
    "start_hour": 360,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1734",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_16_138,LEG_16_210,LEG_16_43"
  },
  {
    "id": 1735,
    "start_hour": 383,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1735",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_17_212,LEG_17_221,LEG_17_170"
  },
  {
    "id": 1736,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1736",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_16_33,LEG_16_36"
  },
  {
    "id": 1737,
    "start_hour": 367,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1737",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_16_113,LEG_16_116"
  },
  {
    "id": 1738,
    "start_hour": 367,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1738",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_16_145,LEG_16_149"
  },
  {
    "id": 1739,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1739",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_16_155,LEG_16_102"
  },
  {
    "id": 1740,
    "start_hour": 364,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1740",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_16_92,LEG_16_91"
  },
  {
    "id": 1741,
    "start_hour": 368,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1741",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_16_250,LEG_16_1"
  },
  {
    "id": 1742,
    "start_hour": 365,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1742",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_16_18,LEG_16_25"
  },
  {
    "id": 1743,
    "start_hour": 366,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1743",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_16_97,LEG_16_213,LEG_16_81"
  },
  {
    "id": 1744,
    "start_hour": 387,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1744",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_17_194,LEG_17_195,LEG_17_192,LEG_17_46"
  },
  {
    "id": 1745,
    "start_hour": 407,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1745",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_18_186,LEG_18_226,LEG_18_140"
  },
  {
    "id": 1746,
    "start_hour": 348,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1746",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_16_74,LEG_16_126,LEG_16_237"
  },
  {
    "id": 1747,
    "start_hour": 373,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1747",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_17_13,LEG_17_172,LEG_17_239"
  },
  {
    "id": 1748,
    "start_hour": 410,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1748",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_18_242,LEG_18_180,LEG_18_6"
  },
  {
    "id": 1749,
    "start_hour": 371,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1749",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_16_31"
  },
  {
    "id": 1750,
    "start_hour": 395,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1750",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_17_36"
  },
  {
    "id": 1751,
    "start_hour": 365,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1751",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_16_164,LEG_16_165"
  },
  {
    "id": 1752,
    "start_hour": 372,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D1752",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_17_28,LEG_17_69,LEG_17_68"
  },
  {
    "id": 1753,
    "start_hour": 412,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1753",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_18_20,LEG_18_132"
  },
  {
    "id": 1754,
    "start_hour": 421,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1754",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_19_122"
  },
  {
    "id": 1755,
    "start_hour": 369,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1755",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_16_159,LEG_16_214"
  },
  {
    "id": 1756,
    "start_hour": 374,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1756",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_17_24,LEG_17_66"
  },
  {
    "id": 1757,
    "start_hour": 397,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1757",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_18_48,LEG_18_50,LEG_18_65"
  },
  {
    "id": 1758,
    "start_hour": 434,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1758",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_19_16"
  },
  {
    "id": 1759,
    "start_hour": 361,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1759",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_16_169,LEG_16_220,LEG_16_188"
  },
  {
    "id": 1760,
    "start_hour": 385,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1760",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_17_40,LEG_17_86"
  },
  {
    "id": 1761,
    "start_hour": 416,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1761",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_18_88,LEG_18_100"
  },
  {
    "id": 1762,
    "start_hour": 432,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1762",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_19_24,LEG_19_121,LEG_19_124"
  },
  {
    "id": 1763,
    "start_hour": 365,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1763",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_16_152,LEG_16_147,LEG_16_231"
  },
  {
    "id": 1764,
    "start_hour": 384,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1764",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_17_238,LEG_17_173"
  },
  {
    "id": 1765,
    "start_hour": 408,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1765",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_18_174,LEG_18_11"
  },
  {
    "id": 1766,
    "start_hour": 428,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1766",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_19_10,LEG_19_62,LEG_19_127"
  },
  {
    "id": 1767,
    "start_hour": 350,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1767",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_16_243,LEG_16_240"
  },
  {
    "id": 1768,
    "start_hour": 372,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1768",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_17_146,LEG_17_200,LEG_17_201"
  },
  {
    "id": 1769,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1769",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_18_72,LEG_18_93,LEG_18_175"
  },
  {
    "id": 1770,
    "start_hour": 276,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D1770",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_13_25,LEG_13_224,LEG_13_63"
  },
  {
    "id": 1771,
    "start_hour": 316,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1771",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_14_22,LEG_14_256"
  },
  {
    "id": 1772,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1772",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_15_154,LEG_15_73,LEG_15_156"
  },
  {
    "id": 1773,
    "start_hour": 289,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1773",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_13_158,LEG_13_159"
  },
  {
    "id": 1774,
    "start_hour": 279,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1774",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_13_140,LEG_13_173"
  },
  {
    "id": 1775,
    "start_hour": 291,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1775",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_13_91,LEG_13_94"
  },
  {
    "id": 1776,
    "start_hour": 289,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1776",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_13_72,LEG_13_238"
  },
  {
    "id": 1777,
    "start_hour": 288,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1777",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_13_240,LEG_13_241"
  },
  {
    "id": 1778,
    "start_hour": 288,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1778",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_13_127,LEG_13_129"
  },
  {
    "id": 1779,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1779",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_13_14,LEG_13_20"
  },
  {
    "id": 1780,
    "start_hour": 276,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1780",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_13_68,LEG_13_65"
  },
  {
    "id": 1781,
    "start_hour": 292,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1781",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_13_71,LEG_13_109"
  },
  {
    "id": 1782,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1782",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_14_208,LEG_14_9"
  },
  {
    "id": 1783,
    "start_hour": 295,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1783",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_13_135,LEG_13_139"
  },
  {
    "id": 1784,
    "start_hour": 296,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1784",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_13_239,LEG_13_1"
  },
  {
    "id": 1785,
    "start_hour": 293,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1785",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_13_16,LEG_13_22"
  },
  {
    "id": 1786,
    "start_hour": 292,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1786",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_13_87,LEG_13_86"
  },
  {
    "id": 1787,
    "start_hour": 287,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1787",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_13_244,LEG_13_112"
  },
  {
    "id": 1788,
    "start_hour": 308,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1788",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_14_70,LEG_14_44,LEG_14_168,LEG_14_49"
  },
  {
    "id": 1789,
    "start_hour": 335,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1789",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_15_212,LEG_15_221,LEG_15_170"
  },
  {
    "id": 1790,
    "start_hour": 278,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1790",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_13_21,LEG_13_61"
  },
  {
    "id": 1791,
    "start_hour": 301,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1791",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_14_48,LEG_14_50,LEG_14_64"
  },
  {
    "id": 1792,
    "start_hour": 338,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1792",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_15_16"
  },
  {
    "id": 1793,
    "start_hour": 299,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1793",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_13_28"
  },
  {
    "id": 1794,
    "start_hour": 323,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1794",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_14_37"
  },
  {
    "id": 1795,
    "start_hour": 293,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1795",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_13_154,LEG_13_155,LEG_13_75"
  },
  {
    "id": 1796,
    "start_hour": 315,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1796",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_14_195,LEG_14_198"
  },
  {
    "id": 1797,
    "start_hour": 336,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1797",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_15_110,LEG_15_41"
  },
  {
    "id": 1798,
    "start_hour": 362,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1798",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_16_32"
  },
  {
    "id": 1799,
    "start_hour": 278,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1799",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_13_232,LEG_13_229"
  },
  {
    "id": 1800,
    "start_hour": 300,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D1800",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_14_30,LEG_14_68,LEG_14_67"
  },
  {
    "id": 1801,
    "start_hour": 340,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1801",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_15_20,LEG_15_132"
  },
  {
    "id": 1802,
    "start_hour": 349,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1802",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_16_131"
  },
  {
    "id": 1803,
    "start_hour": 298,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1803",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_13_171"
  },
  {
    "id": 1804,
    "start_hour": 306,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1804",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_14_248,LEG_14_235"
  },
  {
    "id": 1805,
    "start_hour": 326,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1805",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_15_33,LEG_15_36"
  },
  {
    "id": 1806,
    "start_hour": 297,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1806",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_13_149,LEG_13_207"
  },
  {
    "id": 1807,
    "start_hour": 302,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1807",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_14_158,LEG_14_161,LEG_14_142"
  },
  {
    "id": 1808,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1808",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_15_143,LEG_15_93,LEG_15_175"
  },
  {
    "id": 1809,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1809",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_13_146,LEG_13_97"
  },
  {
    "id": 1810,
    "start_hour": 303,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1810",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_14_134,LEG_14_227,LEG_14_90"
  },
  {
    "id": 1811,
    "start_hour": 344,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1811",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_15_88,LEG_15_81"
  },
  {
    "id": 1812,
    "start_hour": 363,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1812",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_16_194,LEG_16_197,LEG_16_104"
  },
  {
    "id": 1813,
    "start_hour": 293,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1813",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_13_126,LEG_13_125"
  },
  {
    "id": 1814,
    "start_hour": 300,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1814",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_14_167,LEG_14_186"
  },
  {
    "id": 1815,
    "start_hour": 326,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1815",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_15_101,LEG_15_94,LEG_15_253"
  },
  {
    "id": 1816,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1816",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_16_154,LEG_16_73,LEG_16_156"
  },
  {
    "id": 1817,
    "start_hour": 1,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1817",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_01_160"
  },
  {
    "id": 1818,
    "start_hour": -1,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1818",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_01_202,LEG_01_100,LEG_01_94"
  },
  {
    "id": 1819,
    "start_hour": 24,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1819",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_02_111,LEG_02_87"
  },
  {
    "id": 1820,
    "start_hour": 56,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1820",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_03_89,LEG_03_82"
  },
  {
    "id": 1821,
    "start_hour": 73,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1821",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_04_167,LEG_04_236,LEG_04_229"
  },
  {
    "id": 1822,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1822",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_01_196,LEG_01_19"
  },
  {
    "id": 1823,
    "start_hour": 14,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1823",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_02_106,LEG_02_165,LEG_02_236,LEG_02_138,LEG_02_101"
  },
  {
    "id": 1824,
    "start_hour": 47,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1824",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_03_217,LEG_03_67"
  },
  {
    "id": 1825,
    "start_hour": 66,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1825",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_04_63"
  },
  {
    "id": 1826,
    "start_hour": 11,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1826",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_01_28"
  },
  {
    "id": 1827,
    "start_hour": 26,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1827",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_02_188,LEG_02_42"
  },
  {
    "id": 1828,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1828",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_03_33,LEG_03_166"
  },
  {
    "id": 1829,
    "start_hour": 71,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1829",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_04_15,LEG_04_188"
  },
  {
    "id": 1830,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1830",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_01_75"
  },
  {
    "id": 1831,
    "start_hour": 33,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1831",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_02_58"
  },
  {
    "id": 1832,
    "start_hour": 39,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1832",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_03_210,LEG_03_6,LEG_03_22"
  },
  {
    "id": 1833,
    "start_hour": 76,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1833",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_04_64"
  },
  {
    "id": 1834,
    "start_hour": 5,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1834",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_01_72"
  },
  {
    "id": 1835,
    "start_hour": 32,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1835",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_02_89,LEG_02_75"
  },
  {
    "id": 1836,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1836",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_03_18,LEG_03_139,LEG_03_237,LEG_03_181"
  },
  {
    "id": 1837,
    "start_hour": 71,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1837",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_04_249,LEG_04_218,LEG_04_246,LEG_04_245,LEG_04_39"
  },
  {
    "id": 1838,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1838",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_01_49"
  },
  {
    "id": 1839,
    "start_hour": 26,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1839",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_02_17,LEG_02_166"
  },
  {
    "id": 1840,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1840",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_03_15,LEG_03_28"
  },
  {
    "id": 1841,
    "start_hour": 66,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1841",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_04_40,LEG_04_222,LEG_04_230"
  },
  {
    "id": 1842,
    "start_hour": 4,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1842",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_01_200,LEG_01_116"
  },
  {
    "id": 1843,
    "start_hour": 12,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1843",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_02_170,LEG_02_189"
  },
  {
    "id": 1844,
    "start_hour": 38,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1844",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_03_102,LEG_03_95,LEG_03_175"
  },
  {
    "id": 1845,
    "start_hour": 71,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1845",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_04_104,LEG_04_250,LEG_04_54"
  },
  {
    "id": 1846,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1846",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_01_67"
  },
  {
    "id": 1847,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1847",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_02_72,LEG_02_175"
  },
  {
    "id": 1848,
    "start_hour": 47,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1848",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_03_104,LEG_03_250,LEG_03_54"
  },
  {
    "id": 1849,
    "start_hour": 5,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1849",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_01_32"
  },
  {
    "id": 1850,
    "start_hour": 26,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1850",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_02_33,LEG_02_185"
  },
  {
    "id": 1851,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1851",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_03_109,LEG_03_119,LEG_03_224"
  },
  {
    "id": 1852,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1852",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_01_102,LEG_01_98"
  },
  {
    "id": 1853,
    "start_hour": 14,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1853",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_02_161,LEG_02_164"
  },
  {
    "id": 1854,
    "start_hour": 36,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1854",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_03_9,LEG_03_157,LEG_03_236,LEG_03_229"
  },
  {
    "id": 1855,
    "start_hour": 3,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1855",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_01_47,LEG_01_74,LEG_01_11,LEG_01_96"
  },
  {
    "id": 1856,
    "start_hour": 13,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D1856",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_02_142,LEG_02_27,LEG_02_28"
  },
  {
    "id": 1857,
    "start_hour": 42,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1857",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_03_40,LEG_03_222,LEG_03_230"
  },
  {
    "id": 1858,
    "start_hour": 6,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1858",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_01_143,LEG_01_64"
  },
  {
    "id": 1859,
    "start_hour": 12,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D1859",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_02_29,LEG_02_69,LEG_02_38"
  },
  {
    "id": 1860,
    "start_hour": 52,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1860",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_03_43"
  },
  {
    "id": 1861,
    "start_hour": 5,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1861",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_01_44,LEG_01_213"
  },
  {
    "id": 1862,
    "start_hour": 26,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1862",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_02_248"
  },
  {
    "id": 1863,
    "start_hour": 42,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1863",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_03_56,LEG_03_233,LEG_03_118"
  },
  {
    "id": 1864,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1864",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_01_43"
  },
  {
    "id": 1865,
    "start_hour": 30,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1865",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_02_3,LEG_02_22"
  },
  {
    "id": 1866,
    "start_hour": 52,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1866",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_03_64"
  },
  {
    "id": 1867,
    "start_hour": 6,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1867",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_01_107"
  },
  {
    "id": 1868,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1868",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_02_211,LEG_02_8"
  },
  {
    "id": 1869,
    "start_hour": 49,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1869",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_03_186,LEG_03_84"
  },
  {
    "id": 1870,
    "start_hour": 6,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1870",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_01_58,LEG_01_146"
  },
  {
    "id": 1871,
    "start_hour": 23,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1871",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_02_104,LEG_02_67"
  },
  {
    "id": 1872,
    "start_hour": 42,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1872",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_03_63"
  },
  {
    "id": 1873,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1873",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_01_91,LEG_01_154"
  },
  {
    "id": 1874,
    "start_hour": 23,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1874",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_02_109,LEG_02_119,LEG_02_224"
  },
  {
    "id": 1875,
    "start_hour": 2,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1875",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_01_216,LEG_01_42,LEG_01_161"
  },
  {
    "id": 1876,
    "start_hour": 25,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1876",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_02_41,LEG_02_245,LEG_02_39"
  },
  {
    "id": 1877,
    "start_hour": 11,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1877",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_01_165"
  },
  {
    "id": 1878,
    "start_hour": 28,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1878",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_02_193"
  },
  {
    "id": 1879,
    "start_hour": 11,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1879",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_01_29"
  },
  {
    "id": 1880,
    "start_hour": 28,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1880",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_02_43"
  },
  {
    "id": 1881,
    "start_hour": 1,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1881",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_01_51"
  },
  {
    "id": 1882,
    "start_hour": 18,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1882",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_02_63"
  },
  {
    "id": 1883,
    "start_hour": -1,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1883",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_01_65"
  },
  {
    "id": 1884,
    "start_hour": 20,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1884",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_02_70,LEG_02_186,LEG_02_84"
  },
  {
    "id": 1885,
    "start_hour": 1,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1885",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_01_194,LEG_01_119,LEG_01_0"
  },
  {
    "id": 1886,
    "start_hour": 22,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1886",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_02_226,LEG_02_250,LEG_02_54"
  },
  {
    "id": 1887,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1887",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_01_163"
  },
  {
    "id": 1888,
    "start_hour": -1,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1888",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_01_40,LEG_01_41"
  },
  {
    "id": 1889,
    "start_hour": 1,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1889",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_01_182,LEG_01_181"
  },
  {
    "id": 1890,
    "start_hour": -1,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1890",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_01_190,LEG_01_199,LEG_01_212,LEG_01_30"
  },
  {
    "id": 1891,
    "start_hour": -1,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1891",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_01_78,LEG_01_70,LEG_01_38,LEG_01_36"
  },
  {
    "id": 1892,
    "start_hour": 2,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1892",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_01_101,LEG_01_192,LEG_01_180,LEG_01_69"
  },
  {
    "id": 1893,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1893",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_01_50"
  },
  {
    "id": 1894,
    "start_hour": 514,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1894",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_22_256"
  },
  {
    "id": 1895,
    "start_hour": 517,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1895",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_23_217,LEG_23_218,LEG_23_49,LEG_23_30,LEG_23_231"
  },
  {
    "id": 1896,
    "start_hour": 552,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1896",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_24_238,LEG_24_173"
  },
  {
    "id": 1897,
    "start_hour": 576,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1897",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_25_174,LEG_25_144,LEG_25_203"
  },
  {
    "id": 1898,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1898",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_22_202,LEG_22_148"
  },
  {
    "id": 1899,
    "start_hour": 518,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1899",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_23_176,LEG_23_178"
  },
  {
    "id": 1900,
    "start_hour": 540,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1900",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_24_166,LEG_24_185"
  },
  {
    "id": 1901,
    "start_hour": 566,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1901",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_25_101,LEG_25_94,LEG_25_199"
  },
  {
    "id": 1902,
    "start_hour": 514,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1902",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_22_237"
  },
  {
    "id": 1903,
    "start_hour": 517,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1903",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_23_48,LEG_23_50,LEG_23_65"
  },
  {
    "id": 1904,
    "start_hour": 554,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1904",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_24_16,LEG_24_171,LEG_24_199"
  },
  {
    "id": 1905,
    "start_hour": 494,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1905",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_22_58,LEG_22_95"
  },
  {
    "id": 1906,
    "start_hour": 518,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1906",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_23_234"
  },
  {
    "id": 1907,
    "start_hour": 554,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1907",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_24_233,LEG_24_19,LEG_24_196"
  },
  {
    "id": 1908,
    "start_hour": 495,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1908",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_22_206,LEG_22_103"
  },
  {
    "id": 1909,
    "start_hour": 517,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1909",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_23_191,LEG_23_124"
  },
  {
    "id": 1910,
    "start_hour": 494,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1910",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_22_121,LEG_22_79"
  },
  {
    "id": 1911,
    "start_hour": 533,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1911",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_23_56,LEG_23_245"
  },
  {
    "id": 1912,
    "start_hour": 506,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1912",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_22_205,LEG_22_204"
  },
  {
    "id": 1913,
    "start_hour": 506,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1913",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_22_254,LEG_22_257"
  },
  {
    "id": 1914,
    "start_hour": 242,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1914",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_11_254,LEG_11_257"
  },
  {
    "id": 1915,
    "start_hour": 242,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1915",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_11_207,LEG_11_206"
  },
  {
    "id": 1916,
    "start_hour": 231,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1916",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_11_208,LEG_11_104"
  },
  {
    "id": 1917,
    "start_hour": 253,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1917",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_12_175,LEG_12_114"
  },
  {
    "id": 1918,
    "start_hour": 230,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1918",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_11_122,LEG_11_79"
  },
  {
    "id": 1919,
    "start_hour": 269,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1919",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_12_50,LEG_12_219"
  },
  {
    "id": 1920,
    "start_hour": 251,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1920",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_11_129"
  },
  {
    "id": 1921,
    "start_hour": 270,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1921",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_12_66,LEG_12_6"
  },
  {
    "id": 1922,
    "start_hour": 291,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1922",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_13_4,LEG_13_132"
  },
  {
    "id": 1923,
    "start_hour": 317,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1923",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_14_209"
  },
  {
    "id": 1924,
    "start_hour": 250,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1924",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_11_236"
  },
  {
    "id": 1925,
    "start_hour": 253,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1925",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_12_43,LEG_12_45,LEG_12_44,LEG_12_30"
  },
  {
    "id": 1926,
    "start_hour": 276,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D1926",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_13_2,LEG_13_205,LEG_13_162"
  },
  {
    "id": 1927,
    "start_hour": 312,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1927",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_14_175,LEG_14_145,LEG_14_204"
  },
  {
    "id": 1928,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1928",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_11_204,LEG_11_150"
  },
  {
    "id": 1929,
    "start_hour": 254,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1929",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_12_163,LEG_12_164"
  },
  {
    "id": 1930,
    "start_hour": 276,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1930",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_13_156,LEG_13_177"
  },
  {
    "id": 1931,
    "start_hour": 302,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D1931",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_14_100,LEG_14_93,LEG_14_21,LEG_14_197"
  },
  {
    "id": 1932,
    "start_hour": 606,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1932",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_26_178,LEG_26_175"
  },
  {
    "id": 1933,
    "start_hour": 613,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1933",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_27_31,LEG_27_204,LEG_27_127"
  },
  {
    "id": 1934,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1934",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_28_159,LEG_28_97"
  },
  {
    "id": 1935,
    "start_hour": 660,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1935",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_29_193"
  },
  {
    "id": 1936,
    "start_hour": 611,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1936",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_26_116"
  },
  {
    "id": 1937,
    "start_hour": 631,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1937",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_27_13,LEG_27_46"
  },
  {
    "id": 1938,
    "start_hour": 638,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1938",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_28_53,LEG_28_177,LEG_28_19"
  },
  {
    "id": 1939,
    "start_hour": 660,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1939",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_29_260"
  },
  {
    "id": 1940,
    "start_hour": 610,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1940",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_26_214"
  },
  {
    "id": 1941,
    "start_hour": 613,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1941",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_27_174,LEG_27_222,LEG_27_56"
  },
  {
    "id": 1942,
    "start_hour": 651,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1942",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_28_196"
  },
  {
    "id": 1943,
    "start_hour": 606,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1943",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_26_184,LEG_26_135"
  },
  {
    "id": 1944,
    "start_hour": 615,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1944",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_27_106,LEG_27_99,LEG_27_160"
  },
  {
    "id": 1945,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1945",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_26_111,LEG_26_108"
  },
  {
    "id": 1946,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1946",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_26_232,LEG_26_234"
  },
  {
    "id": 1947,
    "start_hour": 590,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1947",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_26_109,LEG_26_113"
  },
  {
    "id": 1948,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1948",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_26_187,LEG_26_186"
  },
  {
    "id": 1949,
    "start_hour": 129,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1949",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_06_218"
  },
  {
    "id": 1950,
    "start_hour": 110,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1950",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_06_108,LEG_06_112"
  },
  {
    "id": 1951,
    "start_hour": 122,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1951",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_06_193,LEG_06_192"
  },
  {
    "id": 1952,
    "start_hour": 122,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1952",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_06_238,LEG_06_241"
  },
  {
    "id": 1953,
    "start_hour": 123,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1953",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_06_217"
  },
  {
    "id": 1954,
    "start_hour": 128,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1954",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_06_7,LEG_06_214"
  },
  {
    "id": 1955,
    "start_hour": 144,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1955",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_07_240,LEG_07_177"
  },
  {
    "id": 1956,
    "start_hour": 168,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1956",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_08_178,LEG_08_148,LEG_08_207"
  },
  {
    "id": 1957,
    "start_hour": 131,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1957",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_06_69"
  },
  {
    "id": 1958,
    "start_hour": 149,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1958",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_07_57,LEG_07_248"
  },
  {
    "id": 1959,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1959",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_06_190,LEG_06_136"
  },
  {
    "id": 1960,
    "start_hour": 134,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1960",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_07_106,LEG_07_165,LEG_07_20,LEG_07_200"
  },
  {
    "id": 1961,
    "start_hour": 130,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1961",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_06_220"
  },
  {
    "id": 1962,
    "start_hour": 133,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1962",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_07_113,LEG_07_107,LEG_07_108"
  },
  {
    "id": 1963,
    "start_hour": 170,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1963",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_08_99,LEG_08_146"
  },
  {
    "id": 1964,
    "start_hour": 197,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1964",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_09_212"
  },
  {
    "id": 1965,
    "start_hour": 131,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1965",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_06_115"
  },
  {
    "id": 1966,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1966",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_07_127,LEG_07_239"
  },
  {
    "id": 1967,
    "start_hour": 157,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1967",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_08_142,LEG_08_27,LEG_08_188,LEG_08_245,LEG_08_39"
  },
  {
    "id": 1968,
    "start_hour": 197,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1968",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_09_57,LEG_09_247"
  },
  {
    "id": 1969,
    "start_hour": 122,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1969",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_06_195"
  },
  {
    "id": 1970,
    "start_hour": 138,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1970",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_07_203,LEG_07_110"
  },
  {
    "id": 1971,
    "start_hour": 168,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1971",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_08_111,LEG_08_87"
  },
  {
    "id": 1972,
    "start_hour": 201,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1972",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_09_58"
  },
  {
    "id": 1973,
    "start_hour": 158,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1973",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_08_244,LEG_08_241"
  },
  {
    "id": 1974,
    "start_hour": 168,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1974",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_08_253,LEG_08_254"
  },
  {
    "id": 1975,
    "start_hour": 171,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1975",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_08_97,LEG_08_100"
  },
  {
    "id": 1976,
    "start_hour": 169,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1976",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_08_173,LEG_08_174"
  },
  {
    "id": 1977,
    "start_hour": 156,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1977",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_08_74,LEG_08_70"
  },
  {
    "id": 1978,
    "start_hour": 158,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1978",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_08_34,LEG_08_30"
  },
  {
    "id": 1979,
    "start_hour": 160,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1979",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_08_116,LEG_08_12"
  },
  {
    "id": 1980,
    "start_hour": 159,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1980",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_08_154,LEG_08_187"
  },
  {
    "id": 1981,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1981",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_08_16,LEG_08_23"
  },
  {
    "id": 1982,
    "start_hour": 159,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1982",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_08_36,LEG_08_221"
  },
  {
    "id": 1983,
    "start_hour": 176,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1983",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_08_20,LEG_08_200"
  },
  {
    "id": 1984,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1984",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_09_211,LEG_09_8"
  },
  {
    "id": 1985,
    "start_hour": 167,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1985",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_08_257,LEG_08_60"
  },
  {
    "id": 1986,
    "start_hour": 188,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1986",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_09_86,LEG_09_130,LEG_09_137"
  },
  {
    "id": 1987,
    "start_hour": 172,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1987",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_08_77,LEG_08_121"
  },
  {
    "id": 1988,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1988",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_09_124,LEG_09_76"
  },
  {
    "id": 1989,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1989",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_08_78,LEG_08_251,LEG_08_181"
  },
  {
    "id": 1990,
    "start_hour": 191,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1990",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_09_249,LEG_09_215,LEG_09_143"
  },
  {
    "id": 1991,
    "start_hour": 168,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1991",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_08_141,LEG_08_214,LEG_08_44"
  },
  {
    "id": 1992,
    "start_hour": 191,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1992",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_09_216,LEG_09_225,LEG_09_144"
  },
  {
    "id": 1993,
    "start_hour": 172,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1993",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_08_93,LEG_08_92"
  },
  {
    "id": 1994,
    "start_hour": 176,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1994",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_08_252,LEG_08_1"
  },
  {
    "id": 1995,
    "start_hour": 173,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1995",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_08_19,LEG_08_26"
  },
  {
    "id": 1996,
    "start_hour": 175,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1996",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_08_149,LEG_08_153"
  },
  {
    "id": 1997,
    "start_hour": 175,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1997",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_08_114,LEG_08_117"
  },
  {
    "id": 1998,
    "start_hour": 179,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1998",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_08_108"
  },
  {
    "id": 1999,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1999",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_09_99,LEG_09_98,LEG_09_219,LEG_09_181"
  },
  {
    "id": 2000,
    "start_hour": 215,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2000",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_10_245,LEG_10_108,LEG_10_105"
  },
  {
    "id": 2001,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2001",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_08_159,LEG_08_103"
  },
  {
    "id": 2002,
    "start_hour": 183,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2002",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_09_136,LEG_09_228,LEG_09_65"
  },
  {
    "id": 2003,
    "start_hour": 218,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2003",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_10_17"
  },
  {
    "id": 2004,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2004",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_08_45,LEG_08_171,LEG_08_55"
  },
  {
    "id": 2005,
    "start_hour": 198,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2005",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_09_3,LEG_09_7"
  },
  {
    "id": 2006,
    "start_hour": 218,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2006",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_10_5"
  },
  {
    "id": 2007,
    "start_hour": 169,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2007",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_08_186,LEG_08_84,LEG_08_192"
  },
  {
    "id": 2008,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2008",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_09_41,LEG_09_227,LEG_09_24"
  },
  {
    "id": 2009,
    "start_hour": 179,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2009",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_08_32"
  },
  {
    "id": 2010,
    "start_hour": 203,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2010",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_09_37"
  },
  {
    "id": 2011,
    "start_hour": 177,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2011",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_08_163,LEG_08_220"
  },
  {
    "id": 2012,
    "start_hour": 182,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2012",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_09_25,LEG_09_66"
  },
  {
    "id": 2013,
    "start_hour": 205,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D2013",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_10_14,LEG_10_172,LEG_10_236"
  },
  {
    "id": 2014,
    "start_hour": 242,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2014",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_11_241,LEG_11_182,LEG_11_6"
  },
  {
    "id": 2015,
    "start_hour": 177,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2015",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_08_94,LEG_08_179"
  },
  {
    "id": 2016,
    "start_hour": 182,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2016",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_09_235"
  },
  {
    "id": 2017,
    "start_hour": 218,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2017",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_10_230,LEG_10_20,LEG_10_196"
  },
  {
    "id": 2018,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2018",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_11_124,LEG_11_76"
  },
  {
    "id": 2019,
    "start_hour": 173,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2019",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_08_156,LEG_08_151"
  },
  {
    "id": 2020,
    "start_hour": 180,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D2020",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_09_29,LEG_09_69,LEG_09_68"
  },
  {
    "id": 2021,
    "start_hour": 220,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2021",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_10_21,LEG_10_131"
  },
  {
    "id": 2022,
    "start_hour": 229,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2022",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_11_132"
  },
  {
    "id": 2023,
    "start_hour": 173,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2023",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_08_168,LEG_08_169,LEG_08_82"
  },
  {
    "id": 2024,
    "start_hour": 195,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2024",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_09_198,LEG_09_129"
  },
  {
    "id": 2025,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2025",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_10_125,LEG_10_234"
  },
  {
    "id": 2026,
    "start_hour": 229,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D2026",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_11_141,LEG_11_27,LEG_11_131,LEG_11_134"
  },
  {
    "id": 2027,
    "start_hour": 178,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2027",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_08_175,LEG_08_203"
  },
  {
    "id": 2028,
    "start_hour": 183,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2028",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_09_210,LEG_09_6,LEG_09_108"
  },
  {
    "id": 2029,
    "start_hour": 218,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2029",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_10_97,LEG_10_238,LEG_10_35"
  },
  {
    "id": 2030,
    "start_hour": 171,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2030",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_08_240"
  },
  {
    "id": 2031,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2031",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_09_243,LEG_09_145"
  },
  {
    "id": 2032,
    "start_hour": 219,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2032",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_10_143,LEG_10_251"
  },
  {
    "id": 2033,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2033",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_11_156,LEG_11_73,LEG_11_158"
  },
  {
    "id": 2034,
    "start_hour": 179,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2034",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_08_7"
  },
  {
    "id": 2035,
    "start_hour": 194,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2035",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_09_5,LEG_09_146"
  },
  {
    "id": 2036,
    "start_hour": 221,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2036",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_10_208,LEG_10_126"
  },
  {
    "id": 2037,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2037",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_11_200,LEG_11_199,LEG_11_105"
  },
  {
    "id": 2038,
    "start_hour": 80,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2038",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_04_91"
  },
  {
    "id": 2039,
    "start_hour": 105,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2039",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_05_50"
  },
  {
    "id": 2040,
    "start_hour": 111,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2040",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_06_194,LEG_06_4,LEG_06_18"
  },
  {
    "id": 2041,
    "start_hour": 148,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2041",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_07_63"
  },
  {
    "id": 2042,
    "start_hour": 83,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2042",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_04_38"
  },
  {
    "id": 2043,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2043",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_05_4,LEG_05_131"
  },
  {
    "id": 2044,
    "start_hour": 123,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2044",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_06_131,LEG_06_168"
  },
  {
    "id": 2045,
    "start_hour": 143,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2045",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_07_109,LEG_07_119,LEG_07_225"
  },
  {
    "id": 2046,
    "start_hour": 80,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2046",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_04_223"
  },
  {
    "id": 2047,
    "start_hour": 98,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2047",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_05_165,LEG_05_103"
  },
  {
    "id": 2048,
    "start_hour": 118,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2048",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_06_111,LEG_06_231"
  },
  {
    "id": 2049,
    "start_hour": 138,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2049",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_07_56,LEG_07_234,LEG_07_118"
  },
  {
    "id": 2050,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2050",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_04_120,LEG_04_115"
  },
  {
    "id": 2051,
    "start_hour": 86,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2051",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_05_148,LEG_05_150,LEG_05_96"
  },
  {
    "id": 2052,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2052",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_06_89,LEG_06_225,LEG_06_37"
  },
  {
    "id": 2053,
    "start_hour": 81,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2053",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_04_44,LEG_04_35"
  },
  {
    "id": 2054,
    "start_hour": 86,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2054",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_05_218,LEG_05_215"
  },
  {
    "id": 2055,
    "start_hour": 108,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2055",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_06_8,LEG_06_142,LEG_06_140,LEG_06_60"
  },
  {
    "id": 2056,
    "start_hour": 80,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2056",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_04_83"
  },
  {
    "id": 2057,
    "start_hour": 98,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2057",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_05_63"
  },
  {
    "id": 2058,
    "start_hour": 108,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D2058",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_06_199,LEG_06_54,LEG_06_105,LEG_06_207"
  },
  {
    "id": 2059,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2059",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_04_225,LEG_04_144,LEG_04_0"
  },
  {
    "id": 2060,
    "start_hour": 94,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2060",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_05_203,LEG_05_98"
  },
  {
    "id": 2061,
    "start_hour": 120,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2061",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_06_100,LEG_06_228,LEG_06_31"
  },
  {
    "id": 2062,
    "start_hour": 76,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2062",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_04_231,LEG_04_138"
  },
  {
    "id": 2063,
    "start_hour": 84,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D2063",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_05_74,LEG_05_212,LEG_05_106,LEG_05_201"
  },
  {
    "id": 2064,
    "start_hour": 83,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2064",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_04_197"
  },
  {
    "id": 2065,
    "start_hour": 100,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2065",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_05_174"
  },
  {
    "id": 2066,
    "start_hour": 80,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2066",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_04_194"
  },
  {
    "id": 2067,
    "start_hour": 71,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2067",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_04_52,LEG_04_53"
  },
  {
    "id": 2068,
    "start_hour": 73,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2068",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_04_215,LEG_04_214"
  },
  {
    "id": 2069,
    "start_hour": 75,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2069",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_04_61,LEG_04_90,LEG_04_213,LEG_04_85"
  },
  {
    "id": 2070,
    "start_hour": 320,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2070",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_14_191"
  },
  {
    "id": 2071,
    "start_hour": 311,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2071",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_14_51,LEG_14_52"
  },
  {
    "id": 2072,
    "start_hour": 301,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2072",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_14_47,LEG_14_187"
  },
  {
    "id": 2073,
    "start_hour": 313,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2073",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_14_212,LEG_14_211"
  },
  {
    "id": 2074,
    "start_hour": 321,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2074",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_14_14,LEG_14_111"
  },
  {
    "id": 2075,
    "start_hour": 325,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2075",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_15_48,LEG_15_50,LEG_15_49,LEG_15_45"
  },
  {
    "id": 2076,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2076",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_14_215,LEG_14_32,LEG_14_1"
  },
  {
    "id": 2077,
    "start_hour": 334,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2077",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_15_227,LEG_15_248,LEG_15_53"
  },
  {
    "id": 2078,
    "start_hour": 313,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2078",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_14_66"
  },
  {
    "id": 2079,
    "start_hour": 330,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2079",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_15_62"
  },
  {
    "id": 2080,
    "start_hour": 320,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2080",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_14_222"
  },
  {
    "id": 2081,
    "start_hour": 338,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2081",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_15_179,LEG_15_235,LEG_15_228"
  },
  {
    "id": 2082,
    "start_hour": 323,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2082",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_14_194"
  },
  {
    "id": 2083,
    "start_hour": 340,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2083",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_15_189"
  },
  {
    "id": 2084,
    "start_hour": 315,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2084",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_14_60,LEG_14_89,LEG_14_210,LEG_14_85"
  },
  {
    "id": 2085,
    "start_hour": 311,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2085",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_14_81"
  },
  {
    "id": 2086,
    "start_hour": 333,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2086",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_15_125,LEG_15_246"
  },
  {
    "id": 2087,
    "start_hour": 354,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2087",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_16_55,LEG_16_232,LEG_16_117"
  },
  {
    "id": 2088,
    "start_hour": 320,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2088",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_14_83"
  },
  {
    "id": 2089,
    "start_hour": 338,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2089",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_15_122,LEG_15_197"
  },
  {
    "id": 2090,
    "start_hour": 360,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2090",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_16_110,LEG_16_244,LEG_16_38"
  },
  {
    "id": 2091,
    "start_hour": 320,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2091",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_14_54"
  },
  {
    "id": 2092,
    "start_hour": 342,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2092",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_15_3,LEG_15_107"
  },
  {
    "id": 2093,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2093",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_16_98,LEG_16_241,LEG_16_45"
  },
  {
    "id": 2094,
    "start_hour": 318,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2094",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_14_169,LEG_14_80,LEG_14_82"
  },
  {
    "id": 2095,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2095",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_15_194,LEG_15_127"
  },
  {
    "id": 2096,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2096",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_16_198,LEG_16_195,LEG_16_192,LEG_16_46"
  },
  {
    "id": 2097,
    "start_hour": 385,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2097",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_17_163,LEG_17_235,LEG_17_228"
  },
  {
    "id": 2098,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2098",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_14_226,LEG_14_25"
  },
  {
    "id": 2099,
    "start_hour": 326,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2099",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_15_105,LEG_15_161,LEG_15_180,LEG_15_6,LEG_15_0"
  },
  {
    "id": 2100,
    "start_hour": 358,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2100",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_16_227,LEG_16_67"
  },
  {
    "id": 2101,
    "start_hour": 378,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2101",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_17_62"
  },
  {
    "id": 2102,
    "start_hour": 316,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2102",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_14_230,LEG_14_135,LEG_14_99"
  },
  {
    "id": 2103,
    "start_hour": 336,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2103",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_15_26,LEG_15_11"
  },
  {
    "id": 2104,
    "start_hour": 356,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2104",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_16_10,LEG_16_71,LEG_16_137"
  },
  {
    "id": 2105,
    "start_hour": 376,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2105",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_17_2,LEG_17_61"
  },
  {
    "id": 2106,
    "start_hour": 407,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2106",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_18_51,LEG_18_52"
  },
  {
    "id": 2107,
    "start_hour": 416,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2107",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_18_190"
  },
  {
    "id": 2108,
    "start_hour": 409,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2108",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_18_211,LEG_18_210"
  },
  {
    "id": 2109,
    "start_hour": 419,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2109",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_18_193"
  },
  {
    "id": 2110,
    "start_hour": 436,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2110",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_19_172"
  },
  {
    "id": 2111,
    "start_hour": 412,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2111",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_18_230,LEG_18_135"
  },
  {
    "id": 2112,
    "start_hour": 420,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2112",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_19_9,LEG_19_144,LEG_19_126,LEG_19_204"
  },
  {
    "id": 2113,
    "start_hour": 411,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2113",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_18_60,LEG_18_89,LEG_18_209,LEG_18_84"
  },
  {
    "id": 2114,
    "start_hour": 409,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2114",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_18_221,LEG_18_170"
  },
  {
    "id": 2115,
    "start_hour": 431,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2115",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_19_93,LEG_19_59"
  },
  {
    "id": 2116,
    "start_hour": 450,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2116",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_20_52"
  },
  {
    "id": 2117,
    "start_hour": 413,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2117",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_18_41"
  },
  {
    "id": 2118,
    "start_hour": 443,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2118",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_19_0"
  },
  {
    "id": 2119,
    "start_hour": 450,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2119",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_20_45"
  },
  {
    "id": 2120,
    "start_hour": 416,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2120",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_18_223"
  },
  {
    "id": 2121,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2121",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_19_165,LEG_19_106"
  },
  {
    "id": 2122,
    "start_hour": 454,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2122",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_20_209,LEG_20_230,LEG_20_43"
  },
  {
    "id": 2123,
    "start_hour": 417,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2123",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_18_43,LEG_18_34"
  },
  {
    "id": 2124,
    "start_hour": 422,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2124",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_19_215,LEG_19_212"
  },
  {
    "id": 2125,
    "start_hour": 444,
    "duration_hours": 16,
    "required_skill": "A320",
    "gerad_duty_id": "D2125",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_20_8,LEG_20_143"
  },
  {
    "id": 2126,
    "start_hour": 472,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2126",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_21_2,LEG_21_61"
  },
  {
    "id": 2127,
    "start_hour": 416,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2127",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_18_82"
  },
  {
    "id": 2128,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2128",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_19_113,LEG_19_183"
  },
  {
    "id": 2129,
    "start_hour": 453,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2129",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_20_116,LEG_20_229"
  },
  {
    "id": 2130,
    "start_hour": 474,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2130",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_21_55,LEG_21_234,LEG_21_119"
  },
  {
    "id": 2131,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2131",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_18_167,LEG_18_23"
  },
  {
    "id": 2132,
    "start_hour": 422,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2132",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_19_163,LEG_19_164"
  },
  {
    "id": 2133,
    "start_hour": 444,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2133",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_20_135,LEG_20_185,LEG_20_186"
  },
  {
    "id": 2134,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2134",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_21_73,LEG_21_243,LEG_21_45"
  },
  {
    "id": 2135,
    "start_hour": 416,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2135",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_18_90"
  },
  {
    "id": 2136,
    "start_hour": 440,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2136",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_19_75,LEG_19_66"
  },
  {
    "id": 2137,
    "start_hour": 460,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2137",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_20_75,LEG_20_205"
  },
  {
    "id": 2138,
    "start_hour": 482,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2138",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_21_181,LEG_21_237,LEG_21_230"
  },
  {
    "id": 2139,
    "start_hour": 417,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2139",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_18_219,LEG_18_111"
  },
  {
    "id": 2140,
    "start_hour": 421,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2140",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_19_103,LEG_19_96"
  },
  {
    "id": 2141,
    "start_hour": 446,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2141",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_20_96,LEG_20_89,LEG_20_18"
  },
  {
    "id": 2142,
    "start_hour": 484,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2142",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_21_64"
  },
  {
    "id": 2143,
    "start_hour": 527,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2143",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_23_51,LEG_23_52"
  },
  {
    "id": 2144,
    "start_hour": 536,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2144",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_23_190"
  },
  {
    "id": 2145,
    "start_hour": 517,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2145",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_23_47,LEG_23_186"
  },
  {
    "id": 2146,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2146",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_23_226,LEG_23_140,LEG_23_0"
  },
  {
    "id": 2147,
    "start_hour": 550,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2147",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_24_227,LEG_24_248,LEG_24_53"
  },
  {
    "id": 2148,
    "start_hour": 539,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2148",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_23_37"
  },
  {
    "id": 2149,
    "start_hour": 556,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2149",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_24_42"
  },
  {
    "id": 2150,
    "start_hour": 539,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2150",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_23_193"
  },
  {
    "id": 2151,
    "start_hour": 556,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2151",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_24_189"
  },
  {
    "id": 2152,
    "start_hour": 530,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2152",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_23_118,LEG_23_224,LEG_23_209,LEG_23_84"
  },
  {
    "id": 2153,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2153",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_23_119,LEG_23_114"
  },
  {
    "id": 2154,
    "start_hour": 542,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2154",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_24_157,LEG_24_160,LEG_24_21"
  },
  {
    "id": 2155,
    "start_hour": 580,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2155",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_25_64"
  },
  {
    "id": 2156,
    "start_hour": 529,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2156",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_23_211,LEG_23_139,LEG_23_177"
  },
  {
    "id": 2157,
    "start_hour": 551,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2157",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_24_247,LEG_24_67"
  },
  {
    "id": 2158,
    "start_hour": 570,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2158",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_25_62"
  },
  {
    "id": 2159,
    "start_hour": 527,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2159",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_23_80"
  },
  {
    "id": 2160,
    "start_hour": 549,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2160",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_24_125,LEG_24_246"
  },
  {
    "id": 2161,
    "start_hour": 570,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2161",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_25_55,LEG_25_232,LEG_25_117"
  },
  {
    "id": 2162,
    "start_hour": 536,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2162",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_23_54"
  },
  {
    "id": 2163,
    "start_hour": 558,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2163",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_24_3,LEG_24_7"
  },
  {
    "id": 2164,
    "start_hour": 580,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2164",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_25_42"
  },
  {
    "id": 2165,
    "start_hour": 537,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2165",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_23_219,LEG_23_111"
  },
  {
    "id": 2166,
    "start_hour": 541,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2166",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_24_112,LEG_24_106,LEG_24_181"
  },
  {
    "id": 2167,
    "start_hour": 575,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2167",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_25_108,LEG_25_118,LEG_25_224"
  },
  {
    "id": 2168,
    "start_hour": 532,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2168",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_23_230,LEG_23_135"
  },
  {
    "id": 2169,
    "start_hour": 540,
    "duration_hours": 28,
    "required_skill": "A320",
    "gerad_duty_id": "D2169",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_24_87,LEG_24_30,LEG_24_107"
  },
  {
    "id": 2170,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2170",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_25_98,LEG_25_241,LEG_25_45"
  },
  {
    "id": 2171,
    "start_hour": 534,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2171",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_23_244,LEG_23_38"
  },
  {
    "id": 2172,
    "start_hour": 541,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D2172",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_24_13,LEG_24_172,LEG_24_239"
  },
  {
    "id": 2173,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2173",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_25_242,LEG_25_151,LEG_25_69,LEG_25_46"
  },
  {
    "id": 2174,
    "start_hour": 601,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2174",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_26_150,LEG_26_38,LEG_26_102"
  },
  {
    "id": 2175,
    "start_hour": 531,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2175",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_23_60,LEG_23_89,LEG_23_82"
  },
  {
    "id": 2176,
    "start_hour": 554,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2176",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_24_72,LEG_24_151"
  },
  {
    "id": 2177,
    "start_hour": 576,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2177",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_25_158,LEG_25_27"
  },
  {
    "id": 2178,
    "start_hour": 594,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2178",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_26_35"
  },
  {
    "id": 2179,
    "start_hour": 536,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2179",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_23_90"
  },
  {
    "id": 2180,
    "start_hour": 560,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2180",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_24_88,LEG_24_75"
  },
  {
    "id": 2181,
    "start_hour": 577,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2181",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_25_17,LEG_25_136,LEG_25_236,LEG_25_171"
  },
  {
    "id": 2182,
    "start_hour": 599,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2182",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_26_91,LEG_26_227,LEG_26_47"
  },
  {
    "id": 2183,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2183",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_23_167,LEG_23_23"
  },
  {
    "id": 2184,
    "start_hour": 542,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2184",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_24_176,LEG_24_178"
  },
  {
    "id": 2185,
    "start_hour": 564,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2185",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_25_166,LEG_25_185"
  },
  {
    "id": 2186,
    "start_hour": 590,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2186",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_26_89,LEG_26_83,LEG_26_146,LEG_26_211"
  },
  {
    "id": 2187,
    "start_hour": 496,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2187",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_22_115,LEG_22_12"
  },
  {
    "id": 2188,
    "start_hour": 505,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2188",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_22_78,LEG_22_249"
  },
  {
    "id": 2189,
    "start_hour": 504,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2189",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_22_251,LEG_22_252"
  },
  {
    "id": 2190,
    "start_hour": 495,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2190",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_22_35,LEG_22_216"
  },
  {
    "id": 2191,
    "start_hour": 492,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2191",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_22_74,LEG_22_70"
  },
  {
    "id": 2192,
    "start_hour": 507,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2192",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_22_96,LEG_22_99"
  },
  {
    "id": 2193,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2193",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_22_15,LEG_22_22"
  },
  {
    "id": 2194,
    "start_hour": 495,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2194",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_22_150,LEG_22_183"
  },
  {
    "id": 2195,
    "start_hour": 508,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2195",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_22_77,LEG_22_120"
  },
  {
    "id": 2196,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2196",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_23_207,LEG_23_8"
  },
  {
    "id": 2197,
    "start_hour": 504,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2197",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_22_138,LEG_22_210,LEG_22_43"
  },
  {
    "id": 2198,
    "start_hour": 527,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2198",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_23_212,LEG_23_221,LEG_23_170"
  },
  {
    "id": 2199,
    "start_hour": 513,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2199",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_22_159,LEG_22_214"
  },
  {
    "id": 2200,
    "start_hour": 518,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2200",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_23_243,LEG_23_240"
  },
  {
    "id": 2201,
    "start_hour": 494,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2201",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_22_33,LEG_22_29,LEG_22_75"
  },
  {
    "id": 2202,
    "start_hour": 529,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2202",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_23_17,LEG_23_136,LEG_23_236"
  },
  {
    "id": 2203,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2203",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_22_44,LEG_22_168,LEG_22_54"
  },
  {
    "id": 2204,
    "start_hour": 534,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2204",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_23_3"
  },
  {
    "id": 2205,
    "start_hour": 515,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2205",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_22_31"
  },
  {
    "id": 2206,
    "start_hour": 536,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2206",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_23_29"
  },
  {
    "id": 2207,
    "start_hour": 503,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2207",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_22_255,LEG_22_59"
  },
  {
    "id": 2208,
    "start_hour": 524,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2208",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_23_85,LEG_23_129,LEG_23_134"
  },
  {
    "id": 2209,
    "start_hour": 511,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2209",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_22_113,LEG_22_116"
  },
  {
    "id": 2210,
    "start_hour": 509,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2210",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_22_18,LEG_22_25"
  },
  {
    "id": 2211,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2211",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_22_155,LEG_22_102"
  },
  {
    "id": 2212,
    "start_hour": 508,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2212",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_22_92,LEG_22_91"
  },
  {
    "id": 2213,
    "start_hour": 512,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2213",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_22_250,LEG_22_1"
  },
  {
    "id": 2214,
    "start_hour": 511,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2214",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_22_145,LEG_22_149"
  },
  {
    "id": 2215,
    "start_hour": 513,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2215",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_22_93,LEG_22_175"
  },
  {
    "id": 2216,
    "start_hour": 518,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2216",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_23_101,LEG_23_94,LEG_23_132"
  },
  {
    "id": 2217,
    "start_hour": 541,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2217",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_24_131"
  },
  {
    "id": 2218,
    "start_hour": 509,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2218",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_22_152,LEG_22_147,LEG_22_231"
  },
  {
    "id": 2219,
    "start_hour": 528,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2219",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_23_238,LEG_23_173"
  },
  {
    "id": 2220,
    "start_hour": 552,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2220",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_24_174,LEG_24_11"
  },
  {
    "id": 2221,
    "start_hour": 572,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2221",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_25_10,LEG_25_71,LEG_25_137"
  },
  {
    "id": 2222,
    "start_hour": 505,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2222",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_22_169,LEG_22_220,LEG_22_63"
  },
  {
    "id": 2223,
    "start_hour": 526,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2223",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_23_225,LEG_23_187"
  },
  {
    "id": 2224,
    "start_hour": 560,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2224",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_24_190"
  },
  {
    "id": 2225,
    "start_hour": 565,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D2225",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_25_13,LEG_25_172,LEG_25_130,LEG_25_133"
  },
  {
    "id": 2226,
    "start_hour": 509,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2226",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_22_136,LEG_22_236,LEG_22_181"
  },
  {
    "id": 2227,
    "start_hour": 527,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2227",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_23_108,LEG_23_109"
  },
  {
    "id": 2228,
    "start_hour": 552,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2228",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_24_110,LEG_24_41"
  },
  {
    "id": 2229,
    "start_hour": 578,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2229",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_25_32"
  },
  {
    "id": 2230,
    "start_hour": 506,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2230",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_22_184,LEG_22_86"
  },
  {
    "id": 2231,
    "start_hour": 536,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2231",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_23_88,LEG_23_81"
  },
  {
    "id": 2232,
    "start_hour": 555,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2232",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_24_194,LEG_24_202,LEG_24_148"
  },
  {
    "id": 2233,
    "start_hour": 509,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2233",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_22_164,LEG_22_165"
  },
  {
    "id": 2234,
    "start_hour": 516,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D2234",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_23_28,LEG_23_69,LEG_23_68"
  },
  {
    "id": 2235,
    "start_hour": 556,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2235",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_24_20,LEG_24_141"
  },
  {
    "id": 2236,
    "start_hour": 579,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2236",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_25_143,LEG_25_93,LEG_25_175"
  },
  {
    "id": 2237,
    "start_hour": 170,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2237",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_08_209,LEG_08_208"
  },
  {
    "id": 2238,
    "start_hour": 170,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2238",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_08_256,LEG_08_259"
  },
  {
    "id": 2239,
    "start_hour": 159,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2239",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_08_210,LEG_08_104"
  },
  {
    "id": 2240,
    "start_hour": 181,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2240",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_09_195,LEG_09_125"
  },
  {
    "id": 2241,
    "start_hour": 178,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2241",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_08_258"
  },
  {
    "id": 2242,
    "start_hour": 181,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2242",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_09_49,LEG_09_51,LEG_09_50,LEG_09_46,LEG_09_47"
  },
  {
    "id": 2243,
    "start_hour": 215,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2243",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_10_94,LEG_10_173"
  },
  {
    "id": 2244,
    "start_hour": 240,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2244",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_11_176,LEG_11_146,LEG_11_205"
  },
  {
    "id": 2245,
    "start_hour": 158,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2245",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_08_122,LEG_08_79"
  },
  {
    "id": 2246,
    "start_hour": 197,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2246",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_09_87"
  },
  {
    "id": 2247,
    "start_hour": 225,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2247",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_10_58"
  },
  {
    "id": 2248,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2248",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_08_211,LEG_08_8"
  },
  {
    "id": 2249,
    "start_hour": 193,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2249",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_09_45,LEG_09_171,LEG_09_245,LEG_09_39"
  },
  {
    "id": 2250,
    "start_hour": 221,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2250",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_10_57,LEG_10_243"
  },
  {
    "id": 2251,
    "start_hour": 179,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2251",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_08_129"
  },
  {
    "id": 2252,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2252",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_09_127,LEG_09_258"
  },
  {
    "id": 2253,
    "start_hour": 205,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2253",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_10_48,LEG_10_194,LEG_10_201"
  },
  {
    "id": 2254,
    "start_hour": 242,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2254",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_11_72,LEG_11_173,LEG_11_201"
  },
  {
    "id": 2255,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2255",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_08_206,LEG_08_152"
  },
  {
    "id": 2256,
    "start_hour": 182,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2256",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_09_180,LEG_09_182"
  },
  {
    "id": 2257,
    "start_hour": 204,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2257",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_10_166,LEG_10_185"
  },
  {
    "id": 2258,
    "start_hour": 230,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2258",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_11_102,LEG_11_95,LEG_11_20,LEG_11_198"
  },
  {
    "id": 2259,
    "start_hour": 26,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2259",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_02_209,LEG_02_208"
  },
  {
    "id": 2260,
    "start_hour": 26,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2260",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_02_256,LEG_02_259"
  },
  {
    "id": 2261,
    "start_hour": 14,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2261",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_02_122,LEG_02_126"
  },
  {
    "id": 2262,
    "start_hour": 34,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2262",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_02_238"
  },
  {
    "id": 2263,
    "start_hour": 37,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2263",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_03_49,LEG_03_51,LEG_03_50,LEG_03_46"
  },
  {
    "id": 2264,
    "start_hour": 71,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2264",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_04_81"
  },
  {
    "id": 2265,
    "start_hour": 35,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2265",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_02_129"
  },
  {
    "id": 2266,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2266",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_03_127,LEG_03_258"
  },
  {
    "id": 2267,
    "start_hour": 61,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2267",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_04_142,LEG_04_27,LEG_04_148,LEG_04_207"
  },
  {
    "id": 2268,
    "start_hour": 14,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2268",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_02_59,LEG_02_96"
  },
  {
    "id": 2269,
    "start_hour": 38,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2269",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_03_235"
  },
  {
    "id": 2270,
    "start_hour": 74,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2270",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_04_234,LEG_04_146"
  },
  {
    "id": 2271,
    "start_hour": 101,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2271",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_05_192"
  },
  {
    "id": 2272,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2272",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_02_206,LEG_02_152"
  },
  {
    "id": 2273,
    "start_hour": 38,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2273",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_03_106,LEG_03_165,LEG_03_98,LEG_03_219,LEG_03_232"
  },
  {
    "id": 2274,
    "start_hour": 72,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2274",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_04_239,LEG_04_177"
  },
  {
    "id": 2275,
    "start_hour": 96,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2275",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_05_161,LEG_05_134,LEG_05_187"
  },
  {
    "id": 2276,
    "start_hour": 543,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2276",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_24_35,LEG_24_216"
  },
  {
    "id": 2277,
    "start_hour": 543,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2277",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_24_150,LEG_24_183"
  },
  {
    "id": 2278,
    "start_hour": 553,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2278",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_24_78,LEG_24_249"
  },
  {
    "id": 2279,
    "start_hour": 555,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2279",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_24_96,LEG_24_99"
  },
  {
    "id": 2280,
    "start_hour": 552,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2280",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_24_251,LEG_24_252"
  },
  {
    "id": 2281,
    "start_hour": 544,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2281",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_24_115,LEG_24_12"
  },
  {
    "id": 2282,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2282",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_24_15,LEG_24_22"
  },
  {
    "id": 2283,
    "start_hour": 557,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2283",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_24_136,LEG_24_236,LEG_24_132"
  },
  {
    "id": 2284,
    "start_hour": 565,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2284",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_25_131"
  },
  {
    "id": 2285,
    "start_hour": 552,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2285",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_24_138,LEG_24_210,LEG_24_43"
  },
  {
    "id": 2286,
    "start_hour": 575,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2286",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_25_212,LEG_25_226,LEG_25_140"
  },
  {
    "id": 2287,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2287",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_24_255,LEG_24_59"
  },
  {
    "id": 2288,
    "start_hour": 572,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2288",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_25_85,LEG_25_129,LEG_25_134"
  },
  {
    "id": 2289,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2289",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_24_155,LEG_24_102"
  },
  {
    "id": 2290,
    "start_hour": 556,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2290",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_24_92,LEG_24_91"
  },
  {
    "id": 2291,
    "start_hour": 557,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2291",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_24_18,LEG_24_25"
  },
  {
    "id": 2292,
    "start_hour": 559,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2292",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_24_113,LEG_24_116"
  },
  {
    "id": 2293,
    "start_hour": 560,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2293",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_24_250,LEG_24_1"
  },
  {
    "id": 2294,
    "start_hour": 559,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2294",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_24_145,LEG_24_149"
  },
  {
    "id": 2295,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2295",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_24_44,LEG_24_168,LEG_24_223"
  },
  {
    "id": 2296,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2296",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_25_179,LEG_25_180,LEG_25_6,LEG_25_0"
  },
  {
    "id": 2297,
    "start_hour": 598,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2297",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_26_206,LEG_26_205,LEG_26_127"
  },
  {
    "id": 2298,
    "start_hour": 544,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2298",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_24_2,LEG_24_61"
  },
  {
    "id": 2299,
    "start_hour": 565,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2299",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_25_191,LEG_25_124"
  },
  {
    "id": 2300,
    "start_hour": 590,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2300",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_26_52,LEG_26_84"
  },
  {
    "id": 2301,
    "start_hour": 540,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2301",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_24_9,LEG_24_153,LEG_24_77"
  },
  {
    "id": 2302,
    "start_hour": 573,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2302",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_25_125,LEG_25_246"
  },
  {
    "id": 2303,
    "start_hour": 594,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2303",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_26_49,LEG_26_165"
  },
  {
    "id": 2304,
    "start_hour": 542,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2304",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_24_24,LEG_24_66"
  },
  {
    "id": 2305,
    "start_hour": 565,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2305",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_25_48,LEG_25_50,LEG_25_65"
  },
  {
    "id": 2306,
    "start_hour": 602,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2306",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_26_15"
  },
  {
    "id": 2307,
    "start_hour": 540,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2307",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_24_74,LEG_24_126,LEG_24_237"
  },
  {
    "id": 2308,
    "start_hour": 565,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2308",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_25_47,LEG_25_163,LEG_25_4,LEG_25_215"
  },
  {
    "id": 2309,
    "start_hour": 563,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2309",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_24_31"
  },
  {
    "id": 2310,
    "start_hour": 587,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2310",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_25_36"
  },
  {
    "id": 2311,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2311",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_24_184,LEG_24_90"
  },
  {
    "id": 2312,
    "start_hour": 585,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2312",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_25_57"
  },
  {
    "id": 2313,
    "start_hour": 591,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2313",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_26_188,LEG_26_6,LEG_26_19"
  },
  {
    "id": 2314,
    "start_hour": 627,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2314",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_27_21"
  },
  {
    "id": 2315,
    "start_hour": 559,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2315",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_24_241,LEG_24_34"
  },
  {
    "id": 2316,
    "start_hour": 566,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2316",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_25_243,LEG_25_240"
  },
  {
    "id": 2317,
    "start_hour": 588,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2317",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_26_8,LEG_26_141,LEG_26_149"
  },
  {
    "id": 2318,
    "start_hour": 624,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2318",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_27_191,LEG_27_104,LEG_27_102"
  },
  {
    "id": 2319,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2319",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_24_4,LEG_24_215"
  },
  {
    "id": 2320,
    "start_hour": 566,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2320",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_25_176,LEG_25_178"
  },
  {
    "id": 2321,
    "start_hour": 588,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2321",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_26_133,LEG_26_182,LEG_26_183"
  },
  {
    "id": 2322,
    "start_hour": 620,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2322",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_27_110,LEG_27_186,LEG_27_123"
  },
  {
    "id": 2323,
    "start_hour": 557,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2323",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_24_152,LEG_24_147,LEG_24_231"
  },
  {
    "id": 2324,
    "start_hour": 576,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2324",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_25_238,LEG_25_239"
  },
  {
    "id": 2325,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2325",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_26_220,LEG_26_103"
  },
  {
    "id": 2326,
    "start_hour": 623,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2326",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_27_150,LEG_27_210"
  },
  {
    "id": 2327,
    "start_hour": 557,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2327",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_24_164,LEG_24_165"
  },
  {
    "id": 2328,
    "start_hour": 564,
    "duration_hours": 29,
    "required_skill": "A319",
    "gerad_duty_id": "D2328",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_25_28,LEG_25_68"
  },
  {
    "id": 2329,
    "start_hour": 604,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2329",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_26_18,LEG_26_120"
  },
  {
    "id": 2330,
    "start_hour": 613,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2330",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_27_105"
  },
  {
    "id": 2331,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2331",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_24_169,LEG_24_220,LEG_24_63"
  },
  {
    "id": 2332,
    "start_hour": 574,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2332",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_25_225,LEG_25_187"
  },
  {
    "id": 2333,
    "start_hour": 608,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2333",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_26_173"
  },
  {
    "id": 2334,
    "start_hour": 613,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2334",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_27_144"
  },
  {
    "id": 2335,
    "start_hour": 540,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2335",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_24_146,LEG_24_200,LEG_24_201"
  },
  {
    "id": 2336,
    "start_hour": 578,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2336",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_25_72,LEG_25_253"
  },
  {
    "id": 2337,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2337",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_26_142,LEG_26_66,LEG_26_144"
  },
  {
    "id": 2338,
    "start_hour": 561,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2338",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_24_159,LEG_24_214"
  },
  {
    "id": 2339,
    "start_hour": 566,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2339",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_25_234"
  },
  {
    "id": 2340,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2340",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_26_212,LEG_26_213"
  },
  {
    "id": 2341,
    "start_hour": 627,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2341",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_27_193,LEG_27_137,LEG_27_206"
  },
  {
    "id": 2342,
    "start_hour": 541,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2342",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_24_47,LEG_24_186"
  },
  {
    "id": 2343,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2343",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_24_51,LEG_24_52"
  },
  {
    "id": 2344,
    "start_hour": 556,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2344",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_24_230,LEG_24_135"
  },
  {
    "id": 2345,
    "start_hour": 564,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2345",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_25_9,LEG_25_153,LEG_25_235,LEG_25_228"
  },
  {
    "id": 2346,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2346",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_24_226,LEG_24_140,LEG_24_0"
  },
  {
    "id": 2347,
    "start_hour": 574,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2347",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_25_227,LEG_25_248,LEG_25_53"
  },
  {
    "id": 2348,
    "start_hour": 551,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2348",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_24_80"
  },
  {
    "id": 2349,
    "start_hour": 572,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2349",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_25_70,LEG_25_182,LEG_25_83"
  },
  {
    "id": 2350,
    "start_hour": 551,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2350",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_24_232,LEG_24_117,LEG_24_109"
  },
  {
    "id": 2351,
    "start_hour": 576,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2351",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_25_110,LEG_25_244,LEG_25_38"
  },
  {
    "id": 2352,
    "start_hour": 555,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2352",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_24_60,LEG_24_89,LEG_24_209,LEG_24_84"
  },
  {
    "id": 2353,
    "start_hour": 560,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2353",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_24_82"
  },
  {
    "id": 2354,
    "start_hour": 578,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2354",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_25_122,LEG_25_201"
  },
  {
    "id": 2355,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2355",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_26_65,LEG_26_138,LEG_26_63"
  },
  {
    "id": 2356,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2356",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_24_244,LEG_24_38,LEG_24_46"
  },
  {
    "id": 2357,
    "start_hour": 575,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2357",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_25_186,LEG_25_109"
  },
  {
    "id": 2358,
    "start_hour": 600,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2358",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_26_98,LEG_26_223,LEG_26_34"
  },
  {
    "id": 2359,
    "start_hour": 561,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2359",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_24_219,LEG_24_111"
  },
  {
    "id": 2360,
    "start_hour": 565,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2360",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_25_112,LEG_25_106,LEG_25_21"
  },
  {
    "id": 2361,
    "start_hour": 604,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2361",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_26_58"
  },
  {
    "id": 2362,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2362",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_24_211,LEG_24_139,LEG_24_177"
  },
  {
    "id": 2363,
    "start_hour": 575,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2363",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_25_247,LEG_25_67"
  },
  {
    "id": 2364,
    "start_hour": 594,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2364",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_26_56,LEG_26_222,LEG_26_224"
  },
  {
    "id": 2365,
    "start_hour": 563,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2365",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_24_193"
  },
  {
    "id": 2366,
    "start_hour": 584,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2366",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_25_190"
  },
  {
    "id": 2367,
    "start_hour": 589,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2367",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_26_100,LEG_26_94,LEG_26_158"
  },
  {
    "id": 2368,
    "start_hour": 625,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2368",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_27_141,LEG_27_145"
  },
  {
    "id": 2369,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2369",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_24_167,LEG_24_23"
  },
  {
    "id": 2370,
    "start_hour": 566,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2370",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_25_105,LEG_25_161,LEG_25_97,LEG_25_213,LEG_25_181"
  },
  {
    "id": 2371,
    "start_hour": 599,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2371",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_26_96,LEG_26_54,LEG_26_78,LEG_26_101"
  },
  {
    "id": 2372,
    "start_hour": 626,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2372",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_27_94,LEG_27_93,LEG_27_57"
  },
  {
    "id": 2373,
    "start_hour": 560,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2373",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_24_65"
  },
  {
    "id": 2374,
    "start_hour": 578,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2374",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_25_16,LEG_25_162"
  },
  {
    "id": 2375,
    "start_hour": 599,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2375",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_26_13,LEG_26_0"
  },
  {
    "id": 2376,
    "start_hour": 619,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2376",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_27_36,LEG_27_32,LEG_27_30"
  },
  {
    "id": 2377,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2377",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_24_119,LEG_24_114"
  },
  {
    "id": 2378,
    "start_hour": 566,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2378",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_25_157,LEG_25_160,LEG_25_141"
  },
  {
    "id": 2379,
    "start_hour": 603,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2379",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_26_130,LEG_26_17"
  },
  {
    "id": 2380,
    "start_hour": 624,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2380",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_27_170,LEG_27_155,LEG_27_82"
  },
  {
    "id": 2381,
    "start_hour": 218,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2381",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_10_205,LEG_10_204"
  },
  {
    "id": 2382,
    "start_hour": 218,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2382",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_10_252,LEG_10_255"
  },
  {
    "id": 2383,
    "start_hour": 206,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2383",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_10_59,LEG_10_186"
  },
  {
    "id": 2384,
    "start_hour": 229,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2384",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_11_193,LEG_11_125"
  },
  {
    "id": 2385,
    "start_hour": 227,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2385",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_10_127"
  },
  {
    "id": 2386,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2386",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_11_127,LEG_11_256"
  },
  {
    "id": 2387,
    "start_hour": 253,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2387",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_12_129,LEG_12_26,LEG_12_134,LEG_12_186"
  },
  {
    "id": 2388,
    "start_hour": 206,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2388",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_10_120,LEG_10_77"
  },
  {
    "id": 2389,
    "start_hour": 245,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2389",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_11_57,LEG_11_245"
  },
  {
    "id": 2390,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2390",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_10_202,LEG_10_148"
  },
  {
    "id": 2391,
    "start_hour": 230,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2391",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_11_178,LEG_11_180,LEG_11_75"
  },
  {
    "id": 2392,
    "start_hour": 268,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2392",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_12_76,LEG_12_75"
  },
  {
    "id": 2393,
    "start_hour": 290,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2393",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_13_111"
  },
  {
    "id": 2394,
    "start_hour": 226,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2394",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_10_254"
  },
  {
    "id": 2395,
    "start_hour": 229,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2395",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_11_49,LEG_11_51,LEG_11_221"
  },
  {
    "id": 2396,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2396",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_12_165,LEG_12_20"
  },
  {
    "id": 2397,
    "start_hour": 287,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2397",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_13_90,LEG_13_134,LEG_13_194"
  },
  {
    "id": 2398,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2398",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_10_207,LEG_10_8"
  },
  {
    "id": 2399,
    "start_hour": 241,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2399",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_11_45,LEG_11_169,LEG_11_190"
  },
  {
    "id": 2400,
    "start_hour": 265,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2400",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_12_38,LEG_12_78"
  },
  {
    "id": 2401,
    "start_hour": 297,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2401",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_13_53"
  },
  {
    "id": 2402,
    "start_hour": 578,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2402",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_25_123"
  },
  {
    "id": 2403,
    "start_hour": 588,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2403",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_26_76,LEG_26_215,LEG_26_106,LEG_26_202,LEG_26_57"
  },
  {
    "id": 2404,
    "start_hour": 627,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2404",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_27_34,LEG_27_221"
  },
  {
    "id": 2405,
    "start_hour": 658,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2405",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_28_46"
  },
  {
    "id": 2406,
    "start_hour": 566,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2406",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_25_121,LEG_25_76"
  },
  {
    "id": 2407,
    "start_hour": 588,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2407",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_26_153,LEG_26_167"
  },
  {
    "id": 2408,
    "start_hour": 614,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2408",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_27_130,LEG_27_128,LEG_27_76"
  },
  {
    "id": 2409,
    "start_hour": 636,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2409",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_28_192"
  },
  {
    "id": 2410,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2410",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_25_202,LEG_25_148"
  },
  {
    "id": 2411,
    "start_hour": 590,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2411",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_26_93,LEG_26_148,LEG_26_86,LEG_26_193,LEG_26_71"
  },
  {
    "id": 2412,
    "start_hour": 624,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2412",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_27_65,LEG_27_182,LEG_27_178"
  },
  {
    "id": 2413,
    "start_hour": 648,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2413",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_28_64"
  },
  {
    "id": 2414,
    "start_hour": 586,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2414",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_25_256"
  },
  {
    "id": 2415,
    "start_hour": 589,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2415",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_26_41,LEG_26_43,LEG_26_59"
  },
  {
    "id": 2416,
    "start_hour": 629,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2416",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_27_43,LEG_27_42"
  },
  {
    "id": 2417,
    "start_hour": 650,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2417",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_28_32,LEG_28_117,LEG_28_190"
  },
  {
    "id": 2418,
    "start_hour": 566,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2418",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_25_58,LEG_25_95"
  },
  {
    "id": 2419,
    "start_hour": 590,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2419",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_26_221,LEG_26_218"
  },
  {
    "id": 2420,
    "start_hour": 612,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2420",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_27_58,LEG_27_159,LEG_27_203,LEG_27_200"
  },
  {
    "id": 2421,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2421",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_25_207,LEG_25_8"
  },
  {
    "id": 2422,
    "start_hour": 599,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2422",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_26_233,LEG_26_53"
  },
  {
    "id": 2423,
    "start_hour": 620,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2423",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_27_217,LEG_27_212"
  },
  {
    "id": 2424,
    "start_hour": 567,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2424",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_25_206,LEG_25_103"
  },
  {
    "id": 2425,
    "start_hour": 589,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2425",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_26_174,LEG_26_112"
  },
  {
    "id": 2426,
    "start_hour": 578,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2426",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_25_254,LEG_25_257"
  },
  {
    "id": 2427,
    "start_hour": 578,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2427",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_25_205,LEG_25_204"
  },
  {
    "id": 2428,
    "start_hour": 385,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2428",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_17_187"
  },
  {
    "id": 2429,
    "start_hour": 392,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2429",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_17_190"
  },
  {
    "id": 2430,
    "start_hour": 383,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2430",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_17_51,LEG_17_52"
  },
  {
    "id": 2431,
    "start_hour": 373,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2431",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_17_47,LEG_17_186"
  },
  {
    "id": 2432,
    "start_hour": 383,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2432",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_17_232,LEG_17_117,LEG_17_109"
  },
  {
    "id": 2433,
    "start_hour": 408,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2433",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_18_110,LEG_18_244,LEG_18_38"
  },
  {
    "id": 2434,
    "start_hour": 395,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2434",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_17_193"
  },
  {
    "id": 2435,
    "start_hour": 412,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2435",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_18_189"
  },
  {
    "id": 2436,
    "start_hour": 392,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2436",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_17_82"
  },
  {
    "id": 2437,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2437",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_18_122,LEG_18_195,LEG_18_192"
  },
  {
    "id": 2438,
    "start_hour": 395,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2438",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_17_37"
  },
  {
    "id": 2439,
    "start_hour": 412,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2439",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_18_42"
  },
  {
    "id": 2440,
    "start_hour": 387,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2440",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_17_60,LEG_17_89,LEG_17_209,LEG_17_84"
  },
  {
    "id": 2441,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2441",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_17_119,LEG_17_114"
  },
  {
    "id": 2442,
    "start_hour": 398,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2442",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_18_157,LEG_18_160,LEG_18_0"
  },
  {
    "id": 2443,
    "start_hour": 430,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2443",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_19_202,LEG_19_221,LEG_19_46"
  },
  {
    "id": 2444,
    "start_hour": 383,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2444",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_17_80"
  },
  {
    "id": 2445,
    "start_hour": 405,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2445",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_18_125,LEG_18_246"
  },
  {
    "id": 2446,
    "start_hour": 426,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2446",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_19_48,LEG_19_216,LEG_19_218"
  },
  {
    "id": 2447,
    "start_hour": 390,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2447",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_17_244,LEG_17_38"
  },
  {
    "id": 2448,
    "start_hour": 397,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D2448",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_18_13,LEG_18_172,LEG_18_239"
  },
  {
    "id": 2449,
    "start_hour": 434,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2449",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_19_214,LEG_19_141,LEG_19_61"
  },
  {
    "id": 2450,
    "start_hour": 393,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2450",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_17_219,LEG_17_111"
  },
  {
    "id": 2451,
    "start_hour": 397,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2451",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_18_112,LEG_18_106,LEG_18_181"
  },
  {
    "id": 2452,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2452",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_19_100,LEG_19_109,LEG_19_199"
  },
  {
    "id": 2453,
    "start_hour": 388,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2453",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_17_230,LEG_17_135,LEG_17_75"
  },
  {
    "id": 2454,
    "start_hour": 409,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2454",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_18_17,LEG_18_136,LEG_18_236,LEG_18_177"
  },
  {
    "id": 2455,
    "start_hour": 432,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2455",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_19_159,LEG_19_211"
  },
  {
    "id": 2456,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2456",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_20_225,LEG_20_141,LEG_20_59"
  },
  {
    "id": 2457,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2457",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_17_167,LEG_17_23"
  },
  {
    "id": 2458,
    "start_hour": 398,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2458",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_18_105,LEG_18_161,LEG_18_97,LEG_18_213,LEG_18_81"
  },
  {
    "id": 2459,
    "start_hour": 435,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2459",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_19_177,LEG_19_119"
  },
  {
    "id": 2460,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2460",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_20_61,LEG_20_224,LEG_20_36"
  },
  {
    "id": 2461,
    "start_hour": 392,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2461",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_17_90"
  },
  {
    "id": 2462,
    "start_hour": 417,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2462",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_18_57"
  },
  {
    "id": 2463,
    "start_hour": 423,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2463",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_19_188,LEG_19_6,LEG_19_20"
  },
  {
    "id": 2464,
    "start_hour": 460,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2464",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_20_53"
  },
  {
    "id": 2465,
    "start_hour": 392,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2465",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_17_223"
  },
  {
    "id": 2466,
    "start_hour": 410,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2466",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_18_179,LEG_18_162"
  },
  {
    "id": 2467,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2467",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_19_14,LEG_19_25"
  },
  {
    "id": 2468,
    "start_hour": 450,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2468",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_20_31,LEG_20_204,LEG_20_211"
  },
  {
    "id": 2469,
    "start_hour": 568,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2469",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_25_115,LEG_25_12"
  },
  {
    "id": 2470,
    "start_hour": 576,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2470",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_25_138,LEG_25_139"
  },
  {
    "id": 2471,
    "start_hour": 567,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2471",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_25_35,LEG_25_216"
  },
  {
    "id": 2472,
    "start_hour": 567,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2472",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_25_150,LEG_25_183"
  },
  {
    "id": 2473,
    "start_hour": 576,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2473",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_25_251,LEG_25_252"
  },
  {
    "id": 2474,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2474",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_25_15,LEG_25_22"
  },
  {
    "id": 2475,
    "start_hour": 577,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2475",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_25_78,LEG_25_249"
  },
  {
    "id": 2476,
    "start_hour": 579,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2476",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_25_96,LEG_25_99"
  },
  {
    "id": 2477,
    "start_hour": 580,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2477",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_25_77,LEG_25_120"
  },
  {
    "id": 2478,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2478",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_26_189,LEG_26_7"
  },
  {
    "id": 2479,
    "start_hour": 566,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2479",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_25_24,LEG_25_66"
  },
  {
    "id": 2480,
    "start_hour": 589,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2480",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_26_12,LEG_26_168,LEG_26_151,LEG_26_152"
  },
  {
    "id": 2481,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2481",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_25_44,LEG_25_168,LEG_25_54"
  },
  {
    "id": 2482,
    "start_hour": 606,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2482",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_26_3"
  },
  {
    "id": 2483,
    "start_hour": 587,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2483",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_25_31"
  },
  {
    "id": 2484,
    "start_hour": 575,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2484",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_25_255,LEG_25_59"
  },
  {
    "id": 2485,
    "start_hour": 596,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2485",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_26_74,LEG_26_117,LEG_26_122"
  },
  {
    "id": 2486,
    "start_hour": 581,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2486",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_25_18,LEG_25_25"
  },
  {
    "id": 2487,
    "start_hour": 580,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2487",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_25_92,LEG_25_91"
  },
  {
    "id": 2488,
    "start_hour": 583,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2488",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_25_145,LEG_25_149"
  },
  {
    "id": 2489,
    "start_hour": 583,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2489",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_25_113,LEG_25_116"
  },
  {
    "id": 2490,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2490",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_25_155,LEG_25_102"
  },
  {
    "id": 2491,
    "start_hour": 584,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2491",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_25_250,LEG_25_1"
  },
  {
    "id": 2492,
    "start_hour": 581,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2492",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_25_152,LEG_25_147,LEG_25_100"
  },
  {
    "id": 2493,
    "start_hour": 600,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2493",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_26_23,LEG_26_217"
  },
  {
    "id": 2494,
    "start_hour": 627,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2494",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_27_120,LEG_27_47"
  },
  {
    "id": 2495,
    "start_hour": 648,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2495",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_28_24,LEG_28_133,LEG_28_130"
  },
  {
    "id": 2496,
    "start_hour": 577,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2496",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_25_169,LEG_25_220,LEG_25_63"
  },
  {
    "id": 2497,
    "start_hour": 598,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2497",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_26_203,LEG_26_97"
  },
  {
    "id": 2498,
    "start_hour": 625,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2498",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_27_79,LEG_27_28"
  },
  {
    "id": 2499,
    "start_hour": 651,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2499",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_28_143"
  },
  {
    "id": 2500,
    "start_hour": 581,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2500",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_25_164,LEG_25_165"
  },
  {
    "id": 2501,
    "start_hour": 588,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D2501",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_26_25,LEG_26_139,LEG_26_10"
  },
  {
    "id": 2502,
    "start_hour": 630,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2502",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_27_7,LEG_27_205"
  },
  {
    "id": 2503,
    "start_hour": 648,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2503",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_28_82,LEG_28_168,LEG_28_175"
  },
  {
    "id": 2504,
    "start_hour": 578,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2504",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_25_184,LEG_25_86"
  },
  {
    "id": 2505,
    "start_hour": 608,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2505",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_26_77,LEG_26_209"
  },
  {
    "id": 2506,
    "start_hour": 625,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2506",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_27_25,LEG_27_224"
  },
  {
    "id": 2507,
    "start_hour": 645,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2507",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_28_259,LEG_28_131,LEG_28_127"
  },
  {
    "id": 2508,
    "start_hour": 564,
    "duration_hours": 28,
    "required_skill": "A320",
    "gerad_duty_id": "D2508",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_25_74,LEG_25_126,LEG_25_128"
  },
  {
    "id": 2509,
    "start_hour": 611,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2509",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_26_70"
  },
  {
    "id": 2510,
    "start_hour": 628,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2510",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_27_175,LEG_27_116"
  },
  {
    "id": 2511,
    "start_hour": 585,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2511",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_25_159,LEG_25_214"
  },
  {
    "id": 2512,
    "start_hour": 590,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2512",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_26_21,LEG_26_60"
  },
  {
    "id": 2513,
    "start_hour": 613,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2513",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_27_14,LEG_27_183,LEG_27_91,LEG_27_185"
  },
  {
    "id": 2514,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2514",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_12_188,LEG_12_187"
  },
  {
    "id": 2515,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2515",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_12_226,LEG_12_229"
  },
  {
    "id": 2516,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2516",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_12_113,LEG_12_110"
  },
  {
    "id": 2517,
    "start_hour": 275,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2517",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_12_70"
  },
  {
    "id": 2518,
    "start_hour": 293,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2518",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_13_52,LEG_13_235"
  },
  {
    "id": 2519,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2519",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_12_185,LEG_12_138"
  },
  {
    "id": 2520,
    "start_hour": 278,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2520",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_13_100,LEG_13_151,LEG_13_17,LEG_13_188"
  },
  {
    "id": 2521,
    "start_hour": 254,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2521",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_12_52,LEG_12_87"
  },
  {
    "id": 2522,
    "start_hour": 278,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2522",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_13_221"
  },
  {
    "id": 2523,
    "start_hour": 314,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2523",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_14_233,LEG_14_143"
  },
  {
    "id": 2524,
    "start_hour": 341,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2524",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_15_208"
  },
  {
    "id": 2525,
    "start_hour": 275,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2525",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_12_118"
  },
  {
    "id": 2526,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2526",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_13_115,LEG_13_245"
  },
  {
    "id": 2527,
    "start_hour": 301,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D2527",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_14_139,LEG_14_28,LEG_14_174"
  },
  {
    "id": 2528,
    "start_hour": 336,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2528",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_15_174,LEG_15_144,LEG_15_203"
  },
  {
    "id": 2529,
    "start_hour": 254,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2529",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_12_111,LEG_12_73"
  },
  {
    "id": 2530,
    "start_hour": 277,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2530",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_13_105,LEG_13_101,LEG_13_160,LEG_13_191"
  },
  {
    "id": 2531,
    "start_hour": 302,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2531",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_14_69,LEG_14_56,LEG_14_247"
  },
  {
    "id": 2532,
    "start_hour": 605,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2532",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_26_140,LEG_26_134"
  },
  {
    "id": 2533,
    "start_hour": 612,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2533",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_27_27,LEG_27_189,LEG_27_149"
  },
  {
    "id": 2534,
    "start_hour": 636,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2534",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_28_236,LEG_28_212,LEG_28_209"
  },
  {
    "id": 2535,
    "start_hour": 661,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D2535",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_29_222,LEG_29_123,LEG_29_129,LEG_29_28"
  },
  {
    "id": 2536,
    "start_hour": 609,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2536",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_26_82,LEG_26_160"
  },
  {
    "id": 2537,
    "start_hour": 614,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2537",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_27_71,LEG_27_64,LEG_27_3"
  },
  {
    "id": 2538,
    "start_hour": 653,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2538",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_28_77,LEG_28_51"
  },
  {
    "id": 2539,
    "start_hour": 674,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2539",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_29_32,LEG_29_117,LEG_29_234"
  },
  {
    "id": 2540,
    "start_hour": 600,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2540",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_26_125,LEG_26_126"
  },
  {
    "id": 2541,
    "start_hour": 591,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2541",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_26_31,LEG_26_195"
  },
  {
    "id": 2542,
    "start_hour": 591,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2542",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_26_137,LEG_26_166"
  },
  {
    "id": 2543,
    "start_hour": 592,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2543",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_26_105,LEG_26_11"
  },
  {
    "id": 2544,
    "start_hour": 600,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2544",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_26_230,LEG_26_231"
  },
  {
    "id": 2545,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2545",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_26_14,LEG_26_20"
  },
  {
    "id": 2546,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2546",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_26_191,LEG_26_192"
  },
  {
    "id": 2547,
    "start_hour": 603,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2547",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_26_85,LEG_26_88"
  },
  {
    "id": 2548,
    "start_hour": 601,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2548",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_26_69,LEG_26_228"
  },
  {
    "id": 2549,
    "start_hour": 611,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2549",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_26_28"
  },
  {
    "id": 2550,
    "start_hour": 627,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2550",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_27_115"
  },
  {
    "id": 2551,
    "start_hour": 608,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2551",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_26_229,LEG_26_1"
  },
  {
    "id": 2552,
    "start_hour": 605,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2552",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_26_16,LEG_26_22"
  },
  {
    "id": 2553,
    "start_hour": 604,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2553",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_26_81,LEG_26_80"
  },
  {
    "id": 2554,
    "start_hour": 611,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2554",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_26_136"
  },
  {
    "id": 2555,
    "start_hour": 592,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2555",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_26_2,LEG_26_55,LEG_26_33"
  },
  {
    "id": 2556,
    "start_hour": 628,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2556",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_27_2,LEG_27_113"
  },
  {
    "id": 2557,
    "start_hour": 638,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2557",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_28_120"
  },
  {
    "id": 2558,
    "start_hour": 605,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2558",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_26_123,LEG_26_208"
  },
  {
    "id": 2559,
    "start_hour": 613,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2559",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_27_153,LEG_27_132,LEG_27_156"
  },
  {
    "id": 2560,
    "start_hour": 651,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2560",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_28_56"
  },
  {
    "id": 2561,
    "start_hour": 611,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2561",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_26_95"
  },
  {
    "id": 2562,
    "start_hour": 627,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2562",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_27_68,LEG_27_184,LEG_27_140"
  },
  {
    "id": 2563,
    "start_hour": 636,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D2563",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_28_123,LEG_28_240,LEG_28_47"
  },
  {
    "id": 2564,
    "start_hour": 607,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2564",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_26_219,LEG_26_39,LEG_26_207"
  },
  {
    "id": 2565,
    "start_hour": 627,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2565",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_27_96,LEG_27_126"
  },
  {
    "id": 2566,
    "start_hour": 637,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2566",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_28_258,LEG_28_7,LEG_28_134,LEG_28_137"
  },
  {
    "id": 2567,
    "start_hour": 601,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2567",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_26_154,LEG_26_199,LEG_26_171"
  },
  {
    "id": 2568,
    "start_hour": 623,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2568",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_27_40,LEG_27_55"
  },
  {
    "id": 2569,
    "start_hour": 645,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2569",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_28_138,LEG_28_234,LEG_28_152"
  },
  {
    "id": 2570,
    "start_hour": 606,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2570",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_26_4,LEG_26_194"
  },
  {
    "id": 2571,
    "start_hour": 614,
    "duration_hours": 17,
    "required_skill": "A320",
    "gerad_duty_id": "D2571",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_27_142,LEG_27_138"
  },
  {
    "id": 2572,
    "start_hour": 637,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2572",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_28_17,LEG_28_21,LEG_28_231,LEG_28_172"
  },
  {
    "id": 2573,
    "start_hour": 603,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2573",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_26_157"
  },
  {
    "id": 2574,
    "start_hour": 612,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D2574",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_27_176,LEG_27_192,LEG_27_129,LEG_27_188"
  },
  {
    "id": 2575,
    "start_hour": 606,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2575",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_26_143,LEG_26_90"
  },
  {
    "id": 2576,
    "start_hour": 614,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2576",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_27_187"
  },
  {
    "id": 2577,
    "start_hour": 650,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2577",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_28_227,LEG_28_266"
  },
  {
    "id": 2578,
    "start_hour": 669,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2578",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_29_262,LEG_29_132,LEG_29_128"
  },
  {
    "id": 2579,
    "start_hour": 505,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2579",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_22_187"
  },
  {
    "id": 2580,
    "start_hour": 493,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2580",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_22_47,LEG_22_186"
  },
  {
    "id": 2581,
    "start_hour": 503,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2581",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_22_51,LEG_22_52"
  },
  {
    "id": 2582,
    "start_hour": 512,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2582",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_22_190"
  },
  {
    "id": 2583,
    "start_hour": 512,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2583",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_22_223"
  },
  {
    "id": 2584,
    "start_hour": 530,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2584",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_23_179,LEG_23_235,LEG_23_228"
  },
  {
    "id": 2585,
    "start_hour": 510,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2585",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_22_49,LEG_22_30,LEG_22_21"
  },
  {
    "id": 2586,
    "start_hour": 532,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2586",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_23_64"
  },
  {
    "id": 2587,
    "start_hour": 505,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2587",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_22_67"
  },
  {
    "id": 2588,
    "start_hour": 522,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2588",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_23_62"
  },
  {
    "id": 2589,
    "start_hour": 515,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2589",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_22_193"
  },
  {
    "id": 2590,
    "start_hour": 532,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2590",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_23_189"
  },
  {
    "id": 2591,
    "start_hour": 507,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2591",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_22_60,LEG_22_89,LEG_22_209,LEG_22_84"
  },
  {
    "id": 2592,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2592",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_22_119,LEG_22_114"
  },
  {
    "id": 2593,
    "start_hour": 518,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2593",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_23_157,LEG_23_160,LEG_23_181"
  },
  {
    "id": 2594,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2594",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_24_108,LEG_24_118,LEG_24_224"
  },
  {
    "id": 2595,
    "start_hour": 503,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2595",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_22_80"
  },
  {
    "id": 2596,
    "start_hour": 525,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2596",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_23_125,LEG_23_246"
  },
  {
    "id": 2597,
    "start_hour": 546,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2597",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_24_55,LEG_24_222,LEG_24_229"
  },
  {
    "id": 2598,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2598",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_22_221,LEG_22_170,LEG_22_177"
  },
  {
    "id": 2599,
    "start_hour": 527,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2599",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_23_247,LEG_23_67"
  },
  {
    "id": 2600,
    "start_hour": 546,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2600",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_24_62"
  },
  {
    "id": 2601,
    "start_hour": 513,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2601",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_22_219,LEG_22_111"
  },
  {
    "id": 2602,
    "start_hour": 517,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2602",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_23_112,LEG_23_106,LEG_23_75"
  },
  {
    "id": 2603,
    "start_hour": 553,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2603",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_24_17"
  },
  {
    "id": 2604,
    "start_hour": 568,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2604",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_25_2,LEG_25_61"
  },
  {
    "id": 2605,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2605",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_22_167,LEG_22_23"
  },
  {
    "id": 2606,
    "start_hour": 518,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D2606",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_23_105,LEG_23_161,LEG_23_162"
  },
  {
    "id": 2607,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2607",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_24_14,LEG_24_27"
  },
  {
    "id": 2608,
    "start_hour": 570,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2608",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_25_39,LEG_25_222,LEG_25_229"
  },
  {
    "id": 2609,
    "start_hour": 508,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2609",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_22_230,LEG_22_135"
  },
  {
    "id": 2610,
    "start_hour": 516,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2610",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_23_87,LEG_23_34"
  },
  {
    "id": 2611,
    "start_hour": 542,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2611",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_24_243,LEG_24_240"
  },
  {
    "id": 2612,
    "start_hour": 564,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2612",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_25_146,LEG_25_200,LEG_25_195,LEG_25_192"
  },
  {
    "id": 2613,
    "start_hour": 512,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2613",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_22_90"
  },
  {
    "id": 2614,
    "start_hour": 537,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2614",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_23_57"
  },
  {
    "id": 2615,
    "start_hour": 542,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2615",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_24_121,LEG_24_79"
  },
  {
    "id": 2616,
    "start_hour": 515,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2616",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_22_37"
  },
  {
    "id": 2617,
    "start_hour": 530,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2617",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_23_5,LEG_23_142"
  },
  {
    "id": 2618,
    "start_hour": 557,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2618",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_24_208,LEG_24_127"
  },
  {
    "id": 2619,
    "start_hour": 578,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2619",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_25_198,LEG_25_237"
  },
  {
    "id": 2620,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2620",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_29_198,LEG_29_148"
  },
  {
    "id": 2621,
    "start_hour": 688,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D2621",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_30_232,LEG_30_230,LEG_30_80"
  },
  {
    "id": 2622,
    "start_hour": 708,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D2622",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_31_240,LEG_31_239,LEG_31_62,LEG_31_46"
  },
  {
    "id": 2623,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2623",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_29_187,LEG_29_105,LEG_29_51"
  },
  {
    "id": 2624,
    "start_hour": 698,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2624",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_30_32,LEG_30_36,LEG_30_50"
  },
  {
    "id": 2625,
    "start_hour": 720,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2625",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_31_64"
  },
  {
    "id": 2626,
    "start_hour": 679,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2626",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_29_189"
  },
  {
    "id": 2627,
    "start_hour": 686,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D2627",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_30_59,LEG_30_65"
  },
  {
    "id": 2628,
    "start_hour": 726,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2628",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_31_44,LEG_31_253"
  },
  {
    "id": 2629,
    "start_hour": 683,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2629",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_29_55"
  },
  {
    "id": 2630,
    "start_hour": 686,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2630",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_30_39,LEG_30_35,LEG_30_66"
  },
  {
    "id": 2631,
    "start_hour": 723,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2631",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_31_197"
  },
  {
    "id": 2632,
    "start_hour": 684,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2632",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_29_65"
  },
  {
    "id": 2633,
    "start_hour": 702,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2633",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_30_33"
  },
  {
    "id": 2634,
    "start_hour": 725,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2634",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_31_10,LEG_31_202"
  },
  {
    "id": 2635,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2635",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_29_13,LEG_29_11"
  },
  {
    "id": 2636,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2636",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_29_195,LEG_29_196"
  },
  {
    "id": 2637,
    "start_hour": 663,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2637",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_29_45,LEG_29_206"
  },
  {
    "id": 2638,
    "start_hour": 664,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2638",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_29_192,LEG_29_12"
  },
  {
    "id": 2639,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2639",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_29_255,LEG_29_258"
  },
  {
    "id": 2640,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2640",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_29_116,LEG_29_115"
  },
  {
    "id": 2641,
    "start_hour": 663,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2641",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_29_201,LEG_29_200"
  },
  {
    "id": 2642,
    "start_hour": 664,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2642",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_29_16,LEG_29_246"
  },
  {
    "id": 2643,
    "start_hour": 675,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2643",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_29_247,LEG_29_244"
  },
  {
    "id": 2644,
    "start_hour": 684,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2644",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_29_203"
  },
  {
    "id": 2645,
    "start_hour": 705,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2645",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_30_9"
  },
  {
    "id": 2646,
    "start_hour": 709,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2646",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_31_87,LEG_31_30,LEG_31_149,LEG_31_204"
  },
  {
    "id": 2647,
    "start_hour": 362,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2647",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_16_205,LEG_16_204"
  },
  {
    "id": 2648,
    "start_hour": 362,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2648",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_16_254,LEG_16_257"
  },
  {
    "id": 2649,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2649",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_16_123,LEG_16_120"
  },
  {
    "id": 2650,
    "start_hour": 370,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2650",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_16_256"
  },
  {
    "id": 2651,
    "start_hour": 373,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2651",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_17_217,LEG_17_218,LEG_17_49,LEG_17_45"
  },
  {
    "id": 2652,
    "start_hour": 407,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2652",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_18_80"
  },
  {
    "id": 2653,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2653",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_16_202,LEG_16_148"
  },
  {
    "id": 2654,
    "start_hour": 374,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2654",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_17_176,LEG_17_178,LEG_17_231"
  },
  {
    "id": 2655,
    "start_hour": 408,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2655",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_18_238,LEG_18_173"
  },
  {
    "id": 2656,
    "start_hour": 432,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2656",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_19_161,LEG_19_134,LEG_19_185"
  },
  {
    "id": 2657,
    "start_hour": 371,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2657",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_16_128"
  },
  {
    "id": 2658,
    "start_hour": 395,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2658",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_17_79"
  },
  {
    "id": 2659,
    "start_hour": 413,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2659",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_18_56,LEG_18_245"
  },
  {
    "id": 2660,
    "start_hour": 350,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2660",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_16_58,LEG_16_95"
  },
  {
    "id": 2661,
    "start_hour": 374,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2661",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_17_234"
  },
  {
    "id": 2662,
    "start_hour": 410,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2662",
    "gerad_crew_id": "C0273",
    "flight_ids": "LEG_18_233,LEG_18_19,LEG_18_196"
  },
  {
    "id": 2663,
    "start_hour": 64,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2663",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_04_116,LEG_04_12"
  },
  {
    "id": 2664,
    "start_hour": 73,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2664",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_04_173,LEG_04_174"
  },
  {
    "id": 2665,
    "start_hour": 63,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2665",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_04_36,LEG_04_221"
  },
  {
    "id": 2666,
    "start_hour": 63,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2666",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_04_154,LEG_04_187"
  },
  {
    "id": 2667,
    "start_hour": 60,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2667",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_04_74,LEG_04_70"
  },
  {
    "id": 2668,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2668",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_04_16,LEG_04_23"
  },
  {
    "id": 2669,
    "start_hour": 75,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2669",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_04_97,LEG_04_100"
  },
  {
    "id": 2670,
    "start_hour": 72,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2670",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_04_253,LEG_04_254"
  },
  {
    "id": 2671,
    "start_hour": 72,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2671",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_04_141,LEG_04_143"
  },
  {
    "id": 2672,
    "start_hour": 73,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2672",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_04_78,LEG_04_251"
  },
  {
    "id": 2673,
    "start_hour": 76,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2673",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_04_77,LEG_04_121"
  },
  {
    "id": 2674,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2674",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_05_191,LEG_05_7"
  },
  {
    "id": 2675,
    "start_hour": 71,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2675",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_04_257,LEG_04_60"
  },
  {
    "id": 2676,
    "start_hour": 92,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2676",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_05_72,LEG_05_117,LEG_05_124"
  },
  {
    "id": 2677,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2677",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_04_45,LEG_04_171,LEG_04_55"
  },
  {
    "id": 2678,
    "start_hour": 102,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2678",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_05_2"
  },
  {
    "id": 2679,
    "start_hour": 77,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2679",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_04_19,LEG_04_26"
  },
  {
    "id": 2680,
    "start_hour": 79,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2680",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_04_149,LEG_04_153"
  },
  {
    "id": 2681,
    "start_hour": 79,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2681",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_04_114,LEG_04_117"
  },
  {
    "id": 2682,
    "start_hour": 76,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2682",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_04_93,LEG_04_92"
  },
  {
    "id": 2683,
    "start_hour": 80,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2683",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_04_252,LEG_04_1"
  },
  {
    "id": 2684,
    "start_hour": 77,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2684",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_04_156,LEG_04_151"
  },
  {
    "id": 2685,
    "start_hour": 84,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D2685",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_05_26,LEG_05_142,LEG_05_10"
  },
  {
    "id": 2686,
    "start_hour": 116,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2686",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_06_9,LEG_06_68,LEG_06_233"
  },
  {
    "id": 2687,
    "start_hour": 62,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2687",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_04_244,LEG_04_241"
  },
  {
    "id": 2688,
    "start_hour": 84,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2688",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_05_156,LEG_05_169"
  },
  {
    "id": 2689,
    "start_hour": 110,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2689",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_06_216"
  },
  {
    "id": 2690,
    "start_hour": 146,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2690",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_07_235,LEG_07_98,LEG_07_220"
  },
  {
    "id": 2691,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2691",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_04_159,LEG_04_103"
  },
  {
    "id": 2692,
    "start_hour": 87,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2692",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_05_123,LEG_05_205,LEG_05_57"
  },
  {
    "id": 2693,
    "start_hour": 122,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2693",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_06_14,LEG_06_167"
  },
  {
    "id": 2694,
    "start_hour": 143,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2694",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_07_104,LEG_07_226,LEG_07_144"
  },
  {
    "id": 2695,
    "start_hour": 77,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2695",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_04_168,LEG_04_169,LEG_04_82"
  },
  {
    "id": 2696,
    "start_hour": 97,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2696",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_05_153,LEG_05_38,LEG_05_102,LEG_05_173"
  },
  {
    "id": 2697,
    "start_hour": 121,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2697",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_06_33,LEG_06_34"
  },
  {
    "id": 2698,
    "start_hour": 146,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2698",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_07_33"
  },
  {
    "id": 2699,
    "start_hour": 83,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2699",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_04_32"
  },
  {
    "id": 2700,
    "start_hour": 98,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2700",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_05_30,LEG_05_132"
  },
  {
    "id": 2701,
    "start_hour": 125,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2701",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_06_196,LEG_06_240"
  },
  {
    "id": 2702,
    "start_hour": 133,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D2702",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_07_142,LEG_07_27,LEG_07_131,LEG_07_134"
  },
  {
    "id": 2703,
    "start_hour": 62,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2703",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_04_34,LEG_04_30,LEG_04_75"
  },
  {
    "id": 2704,
    "start_hour": 100,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2704",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_05_71,LEG_05_70"
  },
  {
    "id": 2705,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2705",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_06_62,LEG_06_28"
  },
  {
    "id": 2706,
    "start_hour": 62,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2706",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_04_25,LEG_04_66"
  },
  {
    "id": 2707,
    "start_hour": 85,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D2707",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_05_128,LEG_05_24,LEG_05_214"
  },
  {
    "id": 2708,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2708",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_06_226,LEG_06_84,LEG_06_162"
  },
  {
    "id": 2709,
    "start_hour": 81,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2709",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_04_163,LEG_04_220"
  },
  {
    "id": 2710,
    "start_hour": 86,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2710",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_05_89,LEG_05_83,LEG_05_230"
  },
  {
    "id": 2711,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2711",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_06_143,LEG_06_63,LEG_06_145"
  },
  {
    "id": 2712,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2712",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_04_4,LEG_04_135"
  },
  {
    "id": 2713,
    "start_hour": 86,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2713",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_05_163,LEG_05_164"
  },
  {
    "id": 2714,
    "start_hour": 108,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2714",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_06_154,LEG_06_173"
  },
  {
    "id": 2715,
    "start_hour": 135,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2715",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_07_136,LEG_07_229,LEG_07_120,LEG_07_115"
  },
  {
    "id": 2716,
    "start_hour": 73,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2716",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_04_186,LEG_04_84,LEG_04_192"
  },
  {
    "id": 2717,
    "start_hour": 97,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2717",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_05_36,LEG_05_73"
  },
  {
    "id": 2718,
    "start_hour": 128,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2718",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_06_79,LEG_06_72"
  },
  {
    "id": 2719,
    "start_hour": 147,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2719",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_07_198,LEG_07_201,LEG_07_105"
  },
  {
    "id": 2720,
    "start_hour": 464,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2720",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_20_208"
  },
  {
    "id": 2721,
    "start_hour": 457,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2721",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_20_172"
  },
  {
    "id": 2722,
    "start_hour": 445,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2722",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_20_42,LEG_20_77"
  },
  {
    "id": 2723,
    "start_hour": 445,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2723",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_20_72,LEG_20_48"
  },
  {
    "id": 2724,
    "start_hour": 464,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2724",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_20_175"
  },
  {
    "id": 2725,
    "start_hour": 455,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2725",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_20_40,LEG_20_41,LEG_20_173"
  },
  {
    "id": 2726,
    "start_hour": 481,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2726",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_21_40,LEG_21_246,LEG_21_38"
  },
  {
    "id": 2727,
    "start_hour": 457,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2727",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_20_73"
  },
  {
    "id": 2728,
    "start_hour": 476,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2728",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_21_71,LEG_21_184,LEG_21_84"
  },
  {
    "id": 2729,
    "start_hour": 467,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2729",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_20_29"
  },
  {
    "id": 2730,
    "start_hour": 484,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2730",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_21_42"
  },
  {
    "id": 2731,
    "start_hour": 457,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2731",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_20_56"
  },
  {
    "id": 2732,
    "start_hour": 474,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2732",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_21_62"
  },
  {
    "id": 2733,
    "start_hour": 458,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2733",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_20_207"
  },
  {
    "id": 2734,
    "start_hour": 467,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2734",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_20_178"
  },
  {
    "id": 2735,
    "start_hour": 484,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2735",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_21_191,LEG_21_190"
  },
  {
    "id": 2736,
    "start_hour": 505,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2736",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_22_40,LEG_22_244,LEG_22_38"
  },
  {
    "id": 2737,
    "start_hour": 467,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2737",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_20_28"
  },
  {
    "id": 2738,
    "start_hour": 482,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2738",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_21_27"
  },
  {
    "id": 2739,
    "start_hour": 498,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2739",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_22_39,LEG_22_222,LEG_22_229"
  },
  {
    "id": 2740,
    "start_hour": 465,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2740",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_20_194,LEG_20_140"
  },
  {
    "id": 2741,
    "start_hour": 470,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2741",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_21_103,LEG_21_96,LEG_21_0"
  },
  {
    "id": 2742,
    "start_hour": 502,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2742",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_22_227,LEG_22_248,LEG_22_53"
  },
  {
    "id": 2743,
    "start_hour": 464,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2743",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_20_44"
  },
  {
    "id": 2744,
    "start_hour": 486,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2744",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_21_3,LEG_21_7"
  },
  {
    "id": 2745,
    "start_hour": 508,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2745",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_22_42"
  },
  {
    "id": 2746,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2746",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_20_110,LEG_20_107"
  },
  {
    "id": 2747,
    "start_hour": 470,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2747",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_21_159,LEG_21_162,LEG_21_183"
  },
  {
    "id": 2748,
    "start_hour": 503,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2748",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_22_108,LEG_22_118,LEG_22_224"
  },
  {
    "id": 2749,
    "start_hour": 460,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2749",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_20_212,LEG_20_210"
  },
  {
    "id": 2750,
    "start_hour": 469,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D2750",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_21_13,LEG_21_174,LEG_21_11"
  },
  {
    "id": 2751,
    "start_hour": 500,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2751",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_22_10,LEG_22_71,LEG_22_137"
  },
  {
    "id": 2752,
    "start_hour": 520,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2752",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_23_2,LEG_23_61"
  },
  {
    "id": 2753,
    "start_hour": 468,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2753",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_20_27"
  },
  {
    "id": 2754,
    "start_hour": 470,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2754",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_21_33,LEG_21_29,LEG_21_102"
  },
  {
    "id": 2755,
    "start_hour": 504,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2755",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_22_26,LEG_22_27"
  },
  {
    "id": 2756,
    "start_hour": 522,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2756",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_23_39,LEG_23_222,LEG_23_229"
  },
  {
    "id": 2757,
    "start_hour": 459,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2757",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_20_50,LEG_20_82,LEG_20_74"
  },
  {
    "id": 2758,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2758",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_21_124,LEG_21_202"
  },
  {
    "id": 2759,
    "start_hour": 506,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2759",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_22_72,LEG_22_162"
  },
  {
    "id": 2760,
    "start_hour": 527,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2760",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_23_14,LEG_23_184"
  },
  {
    "id": 2761,
    "start_hour": 464,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2761",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_20_83"
  },
  {
    "id": 2762,
    "start_hour": 488,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2762",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_21_89,LEG_21_76"
  },
  {
    "id": 2763,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2763",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_22_17,LEG_22_180,LEG_22_6,LEG_22_0"
  },
  {
    "id": 2764,
    "start_hour": 526,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2764",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_23_227,LEG_23_248,LEG_23_53"
  },
  {
    "id": 2765,
    "start_hour": 150,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2765",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_07_246,LEG_07_39"
  },
  {
    "id": 2766,
    "start_hour": 157,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D2766",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_08_14,LEG_08_176,LEG_08_11"
  },
  {
    "id": 2767,
    "start_hour": 188,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2767",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_09_10,LEG_09_71,LEG_09_140"
  },
  {
    "id": 2768,
    "start_hour": 208,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2768",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_10_2,LEG_10_62"
  },
  {
    "id": 2769,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2769",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_07_228,LEG_07_24"
  },
  {
    "id": 2770,
    "start_hour": 158,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2770",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_08_106,LEG_08_165,LEG_08_236,LEG_08_138,LEG_08_101"
  },
  {
    "id": 2771,
    "start_hour": 191,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2771",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_09_217,LEG_09_67"
  },
  {
    "id": 2772,
    "start_hour": 210,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2772",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_10_63"
  },
  {
    "id": 2773,
    "start_hour": 152,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2773",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_07_83"
  },
  {
    "id": 2774,
    "start_hour": 170,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2774",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_08_72,LEG_08_155"
  },
  {
    "id": 2775,
    "start_hour": 192,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2775",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_09_162,LEG_09_28"
  },
  {
    "id": 2776,
    "start_hour": 210,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2776",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_10_40,LEG_10_218,LEG_10_226"
  },
  {
    "id": 2777,
    "start_hour": 155,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2777",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_07_38"
  },
  {
    "id": 2778,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2778",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_08_5,LEG_08_145"
  },
  {
    "id": 2779,
    "start_hour": 195,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2779",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_09_147,LEG_09_82"
  },
  {
    "id": 2780,
    "start_hour": 217,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2780",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_10_163,LEG_10_232,LEG_10_225"
  },
  {
    "id": 2781,
    "start_hour": 153,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2781",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_07_13,LEG_07_112"
  },
  {
    "id": 2782,
    "start_hour": 157,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2782",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_08_113,LEG_08_107"
  },
  {
    "id": 2783,
    "start_hour": 180,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2783",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_09_9,LEG_09_157,LEG_09_236,LEG_09_229"
  },
  {
    "id": 2784,
    "start_hour": 143,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2784",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_07_81"
  },
  {
    "id": 2785,
    "start_hour": 165,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2785",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_08_126,LEG_08_248"
  },
  {
    "id": 2786,
    "start_hour": 186,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2786",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_09_56,LEG_09_233,LEG_09_118"
  },
  {
    "id": 2787,
    "start_hour": 152,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2787",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_07_55"
  },
  {
    "id": 2788,
    "start_hour": 174,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2788",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_08_3,LEG_08_22"
  },
  {
    "id": 2789,
    "start_hour": 196,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2789",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_09_64"
  },
  {
    "id": 2790,
    "start_hour": 152,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2790",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_07_91"
  },
  {
    "id": 2791,
    "start_hour": 177,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2791",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_08_58"
  },
  {
    "id": 2792,
    "start_hour": 182,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2792",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_09_122,LEG_09_79"
  },
  {
    "id": 2793,
    "start_hour": 145,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2793",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_07_219,LEG_07_31,LEG_07_0"
  },
  {
    "id": 2794,
    "start_hour": 166,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2794",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_08_226,LEG_08_250,LEG_08_54"
  },
  {
    "id": 2795,
    "start_hour": 143,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2795",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_07_223,LEG_07_231,LEG_07_50,LEG_07_46"
  },
  {
    "id": 2796,
    "start_hour": 152,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2796",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_07_194"
  },
  {
    "id": 2797,
    "start_hour": 143,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2797",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_07_52,LEG_07_53"
  },
  {
    "id": 2798,
    "start_hour": 133,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2798",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_07_48,LEG_07_190"
  },
  {
    "id": 2799,
    "start_hour": 147,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2799",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_07_61,LEG_07_90,LEG_07_214,LEG_07_85"
  },
  {
    "id": 2800,
    "start_hour": 410,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2800",
    "gerad_crew_id": "C0274",
    "flight_ids": "LEG_18_205,LEG_18_204"
  },
  {
    "id": 2801,
    "start_hour": 410,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2801",
    "gerad_crew_id": "C0275",
    "flight_ids": "LEG_18_254,LEG_18_257"
  },
  {
    "id": 2802,
    "start_hour": 399,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2802",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_18_206,LEG_18_103"
  },
  {
    "id": 2803,
    "start_hour": 421,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2803",
    "gerad_crew_id": "C0276",
    "flight_ids": "LEG_19_174,LEG_19_115"
  },
  {
    "id": 2804,
    "start_hour": 398,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2804",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_18_58,LEG_18_95"
  },
  {
    "id": 2805,
    "start_hour": 424,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2805",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_19_2,LEG_19_54"
  },
  {
    "id": 2806,
    "start_hour": 445,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2806",
    "gerad_crew_id": "C0277",
    "flight_ids": "LEG_20_176,LEG_20_115"
  },
  {
    "id": 2807,
    "start_hour": 418,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2807",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_18_256"
  },
  {
    "id": 2808,
    "start_hour": 421,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2808",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_19_196,LEG_19_197,LEG_19_42,LEG_19_28,LEG_19_205"
  },
  {
    "id": 2809,
    "start_hour": 456,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2809",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_20_221,LEG_20_159"
  },
  {
    "id": 2810,
    "start_hour": 480,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2810",
    "gerad_crew_id": "C0278",
    "flight_ids": "LEG_21_176,LEG_21_146,LEG_21_204"
  },
  {
    "id": 2811,
    "start_hour": 418,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2811",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_18_237"
  },
  {
    "id": 2812,
    "start_hour": 421,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2812",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_19_41,LEG_19_43,LEG_19_57"
  },
  {
    "id": 2813,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2813",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_20_14,LEG_20_130"
  },
  {
    "id": 2814,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2814",
    "gerad_crew_id": "C0279",
    "flight_ids": "LEG_21_145,LEG_21_173,LEG_21_200"
  },
  {
    "id": 2815,
    "start_hour": 657,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2815",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_28_9"
  },
  {
    "id": 2816,
    "start_hour": 661,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D2816",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_29_87,LEG_29_30,LEG_29_31"
  },
  {
    "id": 2817,
    "start_hour": 691,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2817",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_30_108,LEG_30_235,LEG_30_213"
  },
  {
    "id": 2818,
    "start_hour": 710,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2818",
    "gerad_crew_id": "C0280",
    "flight_ids": "LEG_31_91,LEG_31_200"
  },
  {
    "id": 2819,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2819",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_28_255"
  },
  {
    "id": 2820,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2820",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_29_15,LEG_29_245"
  },
  {
    "id": 2821,
    "start_hour": 686,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2821",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_30_53,LEG_30_177,LEG_30_19"
  },
  {
    "id": 2822,
    "start_hour": 708,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2822",
    "gerad_crew_id": "C0281",
    "flight_ids": "LEG_31_260"
  },
  {
    "id": 2823,
    "start_hour": 651,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2823",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_28_246,LEG_28_243,LEG_28_189"
  },
  {
    "id": 2824,
    "start_hour": 673,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2824",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_29_94,LEG_29_159"
  },
  {
    "id": 2825,
    "start_hour": 699,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2825",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_30_159,LEG_30_97"
  },
  {
    "id": 2826,
    "start_hour": 708,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2826",
    "gerad_crew_id": "C0282",
    "flight_ids": "LEG_31_193"
  },
  {
    "id": 2827,
    "start_hour": 655,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2827",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_28_188"
  },
  {
    "id": 2828,
    "start_hour": 662,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2828",
    "gerad_crew_id": "C0283",
    "flight_ids": "LEG_29_59,LEG_29_10,LEG_29_202"
  },
  {
    "id": 2829,
    "start_hour": 660,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2829",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_28_65"
  },
  {
    "id": 2830,
    "start_hour": 678,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2830",
    "gerad_crew_id": "C0284",
    "flight_ids": "LEG_29_44,LEG_29_253"
  },
  {
    "id": 2831,
    "start_hour": 639,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2831",
    "gerad_crew_id": "C0285",
    "flight_ids": "LEG_28_200,LEG_28_199"
  },
  {
    "id": 2832,
    "start_hour": 639,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2832",
    "gerad_crew_id": "C0286",
    "flight_ids": "LEG_28_45,LEG_28_205"
  },
  {
    "id": 2833,
    "start_hour": 640,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2833",
    "gerad_crew_id": "C0287",
    "flight_ids": "LEG_28_191,LEG_28_12"
  },
  {
    "id": 2834,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2834",
    "gerad_crew_id": "C0288",
    "flight_ids": "LEG_28_253,LEG_28_256"
  },
  {
    "id": 2835,
    "start_hour": 640,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2835",
    "gerad_crew_id": "C0289",
    "flight_ids": "LEG_28_16,LEG_28_245"
  },
  {
    "id": 2836,
    "start_hour": 651,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2836",
    "gerad_crew_id": "C0290",
    "flight_ids": "LEG_28_194,LEG_28_195"
  },
  {
    "id": 2837,
    "start_hour": 651,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2837",
    "gerad_crew_id": "C0291",
    "flight_ids": "LEG_28_13,LEG_28_11"
  },
  {
    "id": 2838,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2838",
    "gerad_crew_id": "C0292",
    "flight_ids": "LEG_28_116,LEG_28_115"
  },
  {
    "id": 2839,
    "start_hour": 659,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2839",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_28_55"
  },
  {
    "id": 2840,
    "start_hour": 662,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2840",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_29_91,LEG_29_89,LEG_29_66"
  },
  {
    "id": 2841,
    "start_hour": 699,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2841",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_30_56,LEG_30_143"
  },
  {
    "id": 2842,
    "start_hour": 721,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2842",
    "gerad_crew_id": "C0293",
    "flight_ids": "LEG_31_212,LEG_31_117"
  },
  {
    "id": 2843,
    "start_hour": 660,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2843",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_28_202"
  },
  {
    "id": 2844,
    "start_hour": 681,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2844",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_29_9"
  },
  {
    "id": 2845,
    "start_hour": 685,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D2845",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_30_137,LEG_30_248,LEG_30_158"
  },
  {
    "id": 2846,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2846",
    "gerad_crew_id": "C0294",
    "flight_ids": "LEG_31_160,LEG_31_143"
  },
  {
    "id": 2847,
    "start_hour": 700,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2847",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_30_168,LEG_30_175"
  },
  {
    "id": 2848,
    "start_hour": 709,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2848",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_31_127,LEG_31_165,LEG_31_110,LEG_31_234"
  },
  {
    "id": 2849,
    "start_hour": 685,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2849",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_30_58,LEG_30_15,LEG_30_55"
  },
  {
    "id": 2850,
    "start_hour": 710,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D2850",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_31_53,LEG_31_178"
  },
  {
    "id": 2851,
    "start_hour": 703,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2851",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_30_129,LEG_30_28"
  },
  {
    "id": 2852,
    "start_hour": 712,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2852",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_31_121,LEG_31_220,LEG_31_90,LEG_31_126"
  },
  {
    "id": 2853,
    "start_hour": 687,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2853",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_30_26,LEG_30_52,LEG_30_223"
  },
  {
    "id": 2854,
    "start_hour": 723,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2854",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_31_37,LEG_31_211,LEG_31_125"
  },
  {
    "id": 2855,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2855",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_30_186,LEG_30_105,LEG_30_51"
  },
  {
    "id": 2856,
    "start_hour": 722,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2856",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_31_32,LEG_31_106,LEG_31_99"
  },
  {
    "id": 2857,
    "start_hour": 704,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2857",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_30_169,LEG_30_250"
  },
  {
    "id": 2858,
    "start_hour": 711,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2858",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_31_175,LEG_31_171,LEG_31_161,LEG_31_238"
  },
  {
    "id": 2859,
    "start_hour": 706,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2859",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_30_181"
  },
  {
    "id": 2860,
    "start_hour": 709,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2860",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_31_122,LEG_31_163,LEG_31_158"
  },
  {
    "id": 2861,
    "start_hour": 698,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2861",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_30_247,LEG_30_2"
  },
  {
    "id": 2862,
    "start_hour": 701,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2862",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_30_10,LEG_30_201"
  },
  {
    "id": 2863,
    "start_hour": 723,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2863",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_31_13,LEG_31_61"
  },
  {
    "id": 2864,
    "start_hour": 708,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2864",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_30_93"
  },
  {
    "id": 2865,
    "start_hour": 723,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2865",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_31_144"
  },
  {
    "id": 2866,
    "start_hour": 703,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2866",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_30_84,LEG_30_226"
  },
  {
    "id": 2867,
    "start_hour": 700,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2867",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_30_25,LEG_30_18"
  },
  {
    "id": 2868,
    "start_hour": 688,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2868",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_30_154,LEG_30_133"
  },
  {
    "id": 2869,
    "start_hour": 697,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2869",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_30_113,LEG_30_74"
  },
  {
    "id": 2870,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2870",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_30_8,LEG_30_6"
  },
  {
    "id": 2871,
    "start_hour": 700,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2871",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_30_83,LEG_30_88"
  },
  {
    "id": 2872,
    "start_hour": 697,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2872",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_30_262,LEG_30_263"
  },
  {
    "id": 2873,
    "start_hour": 704,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2873",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_30_150"
  },
  {
    "id": 2874,
    "start_hour": 708,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2874",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_31_150"
  },
  {
    "id": 2875,
    "start_hour": 705,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2875",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_30_75"
  },
  {
    "id": 2876,
    "start_hour": 710,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2876",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_31_70"
  },
  {
    "id": 2877,
    "start_hour": 702,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2877",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_30_22,LEG_30_23"
  },
  {
    "id": 2878,
    "start_hour": 689,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2878",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_30_111,LEG_30_167"
  },
  {
    "id": 2879,
    "start_hour": 698,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2879",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_30_140,LEG_30_141"
  },
  {
    "id": 2880,
    "start_hour": 705,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2880",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_30_3"
  },
  {
    "id": 2881,
    "start_hour": 709,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2881",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_31_1"
  },
  {
    "id": 2882,
    "start_hour": 702,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2882",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_30_153,LEG_30_155"
  },
  {
    "id": 2883,
    "start_hour": 705,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2883",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_30_178"
  },
  {
    "id": 2884,
    "start_hour": 709,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2884",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_31_180"
  },
  {
    "id": 2885,
    "start_hour": 697,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2885",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_30_266,LEG_30_180"
  },
  {
    "id": 2886,
    "start_hour": 703,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2886",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_30_231,LEG_30_172"
  },
  {
    "id": 2887,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2887",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_31_100,LEG_31_188,LEG_31_198,LEG_31_148"
  },
  {
    "id": 2888,
    "start_hour": 704,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2888",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_30_160,LEG_30_237"
  },
  {
    "id": 2889,
    "start_hour": 712,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2889",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_31_233,LEG_31_231,LEG_31_84,LEG_31_227"
  },
  {
    "id": 2890,
    "start_hour": 685,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2890",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_30_73,LEG_30_72"
  },
  {
    "id": 2891,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2891",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_31_17,LEG_31_21,LEG_31_170,LEG_31_251"
  },
  {
    "id": 2892,
    "start_hour": 687,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2892",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_30_27,LEG_30_101"
  },
  {
    "id": 2893,
    "start_hour": 709,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D2893",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_31_137,LEG_31_249,LEG_31_159"
  },
  {
    "id": 2894,
    "start_hour": 708,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2894",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_30_86"
  },
  {
    "id": 2895,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2895",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_31_85,LEG_31_232,LEG_31_173"
  },
  {
    "id": 2896,
    "start_hour": 687,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2896",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_30_146,LEG_30_81,LEG_30_142"
  },
  {
    "id": 2897,
    "start_hour": 710,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2897",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_31_120"
  },
  {
    "id": 2898,
    "start_hour": 183,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2898",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_09_154,LEG_09_187"
  },
  {
    "id": 2899,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2899",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_09_16,LEG_09_23"
  },
  {
    "id": 2900,
    "start_hour": 183,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2900",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_09_36,LEG_09_221"
  },
  {
    "id": 2901,
    "start_hour": 193,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2901",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_09_173,LEG_09_174"
  },
  {
    "id": 2902,
    "start_hour": 180,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2902",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_09_74,LEG_09_70"
  },
  {
    "id": 2903,
    "start_hour": 193,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2903",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_09_78,LEG_09_251"
  },
  {
    "id": 2904,
    "start_hour": 184,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2904",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_09_116,LEG_09_12"
  },
  {
    "id": 2905,
    "start_hour": 192,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2905",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_09_253,LEG_09_254"
  },
  {
    "id": 2906,
    "start_hour": 195,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2906",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_09_97,LEG_09_100"
  },
  {
    "id": 2907,
    "start_hour": 196,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2907",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_09_77,LEG_09_121"
  },
  {
    "id": 2908,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2908",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_10_122,LEG_10_74"
  },
  {
    "id": 2909,
    "start_hour": 192,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2909",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_09_141,LEG_09_214,LEG_09_44"
  },
  {
    "id": 2910,
    "start_hour": 215,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2910",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_10_212,LEG_10_221,LEG_10_140"
  },
  {
    "id": 2911,
    "start_hour": 191,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2911",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_09_257,LEG_09_60"
  },
  {
    "id": 2912,
    "start_hour": 212,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2912",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_10_84,LEG_10_128,LEG_10_135"
  },
  {
    "id": 2913,
    "start_hour": 199,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2913",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_09_149,LEG_09_153"
  },
  {
    "id": 2914,
    "start_hour": 197,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2914",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_09_19,LEG_09_26"
  },
  {
    "id": 2915,
    "start_hour": 199,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2915",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_09_114,LEG_09_117"
  },
  {
    "id": 2916,
    "start_hour": 196,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2916",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_09_93,LEG_09_92"
  },
  {
    "id": 2917,
    "start_hour": 200,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2917",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_09_252,LEG_09_1"
  },
  {
    "id": 2918,
    "start_hour": 182,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2918",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_09_34,LEG_09_30,LEG_09_75"
  },
  {
    "id": 2919,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2919",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_10_18,LEG_10_137,LEG_10_233,LEG_10_177"
  },
  {
    "id": 2920,
    "start_hour": 239,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2920",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_11_247,LEG_11_223,LEG_11_142"
  },
  {
    "id": 2921,
    "start_hour": 184,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2921",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_09_2,LEG_09_62"
  },
  {
    "id": 2922,
    "start_hour": 205,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2922",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_10_191,LEG_10_123"
  },
  {
    "id": 2923,
    "start_hour": 230,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2923",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_11_59,LEG_11_96"
  },
  {
    "id": 2924,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2924",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_09_159,LEG_09_103"
  },
  {
    "id": 2925,
    "start_hour": 207,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2925",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_10_134,LEG_10_224,LEG_10_65"
  },
  {
    "id": 2926,
    "start_hour": 242,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2926",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_11_17"
  },
  {
    "id": 2927,
    "start_hour": 203,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2927",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_09_32"
  },
  {
    "id": 2928,
    "start_hour": 227,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2928",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_10_37"
  },
  {
    "id": 2929,
    "start_hour": 201,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2929",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_09_163,LEG_09_220"
  },
  {
    "id": 2930,
    "start_hour": 206,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2930",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_10_25,LEG_10_66"
  },
  {
    "id": 2931,
    "start_hour": 229,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D2931",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_11_14,LEG_11_174,LEG_11_11"
  },
  {
    "id": 2932,
    "start_hour": 260,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2932",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_12_9,LEG_12_63,LEG_12_128"
  },
  {
    "id": 2933,
    "start_hour": 196,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2933",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_09_148,LEG_09_207,LEG_09_128"
  },
  {
    "id": 2934,
    "start_hour": 218,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2934",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_10_198,LEG_10_197"
  },
  {
    "id": 2935,
    "start_hour": 240,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2935",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_11_111,LEG_11_42"
  },
  {
    "id": 2936,
    "start_hour": 266,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2936",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_12_32"
  },
  {
    "id": 2937,
    "start_hour": 193,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2937",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_09_186,LEG_09_84,LEG_09_192"
  },
  {
    "id": 2938,
    "start_hour": 217,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2938",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_10_41,LEG_10_85"
  },
  {
    "id": 2939,
    "start_hour": 248,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2939",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_11_89,LEG_11_101"
  },
  {
    "id": 2940,
    "start_hour": 263,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2940",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_12_192,LEG_12_57"
  },
  {
    "id": 2941,
    "start_hour": 202,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2941",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_09_175,LEG_09_203"
  },
  {
    "id": 2942,
    "start_hour": 207,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2942",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_10_206,LEG_10_6,LEG_10_106"
  },
  {
    "id": 2943,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2943",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_11_99,LEG_11_0"
  },
  {
    "id": 2944,
    "start_hour": 262,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2944",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_12_201,LEG_12_200,LEG_12_130"
  },
  {
    "id": 2945,
    "start_hour": 201,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2945",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_09_94,LEG_09_179"
  },
  {
    "id": 2946,
    "start_hour": 206,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2946",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_10_100,LEG_10_93,LEG_10_141"
  },
  {
    "id": 2947,
    "start_hour": 243,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2947",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_11_145,LEG_11_133"
  },
  {
    "id": 2948,
    "start_hour": 253,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2948",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_12_121"
  },
  {
    "id": 2949,
    "start_hour": 182,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2949",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_09_244,LEG_09_241"
  },
  {
    "id": 2950,
    "start_hour": 204,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2950",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_10_146,LEG_10_200,LEG_10_195,LEG_10_192,LEG_10_47"
  },
  {
    "id": 2951,
    "start_hour": 241,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2951",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_11_165,LEG_11_4,LEG_11_135"
  },
  {
    "id": 2952,
    "start_hour": 197,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2952",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_09_156,LEG_09_151,LEG_09_101"
  },
  {
    "id": 2953,
    "start_hour": 215,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2953",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_10_213,LEG_10_187"
  },
  {
    "id": 2954,
    "start_hour": 248,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2954",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_11_192"
  },
  {
    "id": 2955,
    "start_hour": 253,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2955",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_12_15,LEG_12_169,LEG_12_146,LEG_12_93"
  },
  {
    "id": 2956,
    "start_hour": 197,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2956",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_09_168,LEG_09_169"
  },
  {
    "id": 2957,
    "start_hour": 204,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D2957",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_10_29,LEG_10_69,LEG_10_68"
  },
  {
    "id": 2958,
    "start_hour": 244,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2958",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_11_21,LEG_11_253"
  },
  {
    "id": 2959,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2959",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_12_145,LEG_12_65,LEG_12_147"
  },
  {
    "id": 2960,
    "start_hour": 27,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2960",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_02_97,LEG_02_100"
  },
  {
    "id": 2961,
    "start_hour": 25,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2961",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_02_173,LEG_02_174"
  },
  {
    "id": 2962,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2962",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_02_16,LEG_02_23"
  },
  {
    "id": 2963,
    "start_hour": 16,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2963",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_02_116,LEG_02_12"
  },
  {
    "id": 2964,
    "start_hour": 15,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2964",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_02_154,LEG_02_187"
  },
  {
    "id": 2965,
    "start_hour": 25,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2965",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_02_78,LEG_02_251"
  },
  {
    "id": 2966,
    "start_hour": 14,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2966",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_02_244,LEG_02_241"
  },
  {
    "id": 2967,
    "start_hour": 15,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2967",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_02_36,LEG_02_221"
  },
  {
    "id": 2968,
    "start_hour": 24,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2968",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_02_253,LEG_02_254"
  },
  {
    "id": 2969,
    "start_hour": 12,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2969",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_02_74,LEG_02_127,LEG_02_258"
  },
  {
    "id": 2970,
    "start_hour": 37,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D2970",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_03_142,LEG_03_27,LEG_03_131,LEG_03_134"
  },
  {
    "id": 2971,
    "start_hour": 32,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2971",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_02_20,LEG_02_200"
  },
  {
    "id": 2972,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2972",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_03_124,LEG_03_76"
  },
  {
    "id": 2973,
    "start_hour": 24,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2973",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_02_141,LEG_02_214,LEG_02_44"
  },
  {
    "id": 2974,
    "start_hour": 47,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2974",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_03_216,LEG_03_225,LEG_03_144"
  },
  {
    "id": 2975,
    "start_hour": 23,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2975",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_02_257,LEG_02_60"
  },
  {
    "id": 2976,
    "start_hour": 44,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2976",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_03_86,LEG_03_130,LEG_03_137"
  },
  {
    "id": 2977,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2977",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_02_159,LEG_02_103"
  },
  {
    "id": 2978,
    "start_hour": 32,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2978",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_02_252,LEG_02_1"
  },
  {
    "id": 2979,
    "start_hour": 31,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2979",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_02_114,LEG_02_117"
  },
  {
    "id": 2980,
    "start_hour": 29,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2980",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_02_19,LEG_02_26"
  },
  {
    "id": 2981,
    "start_hour": 31,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2981",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_02_149,LEG_02_153"
  },
  {
    "id": 2982,
    "start_hour": 28,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2982",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_02_93,LEG_02_92"
  },
  {
    "id": 2983,
    "start_hour": 12,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2983",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_02_150,LEG_02_204,LEG_02_205"
  },
  {
    "id": 2984,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2984",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_03_72,LEG_03_242,LEG_03_35"
  },
  {
    "id": 2985,
    "start_hour": 35,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2985",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_02_32"
  },
  {
    "id": 2986,
    "start_hour": 56,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2986",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_03_30,LEG_03_75"
  },
  {
    "id": 2987,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2987",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_04_18,LEG_04_139,LEG_04_237,LEG_04_181"
  },
  {
    "id": 2988,
    "start_hour": 96,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2988",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_05_159,LEG_05_118,LEG_05_121"
  },
  {
    "id": 2989,
    "start_hour": 29,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2989",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_02_156,LEG_02_151,LEG_02_232"
  },
  {
    "id": 2990,
    "start_hour": 48,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2990",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_03_239,LEG_03_177"
  },
  {
    "id": 2991,
    "start_hour": 72,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2991",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_04_178,LEG_04_11"
  },
  {
    "id": 2992,
    "start_hour": 92,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2992",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_05_9,LEG_05_62,LEG_05_126"
  },
  {
    "id": 2993,
    "start_hour": 29,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2993",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_02_168,LEG_02_169"
  },
  {
    "id": 2994,
    "start_hour": 36,
    "duration_hours": 29,
    "required_skill": "A320",
    "gerad_duty_id": "D2994",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_03_29,LEG_03_69,LEG_03_68"
  },
  {
    "id": 2995,
    "start_hour": 76,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2995",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_04_21,LEG_04_133"
  },
  {
    "id": 2996,
    "start_hour": 85,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2996",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_05_119"
  },
  {
    "id": 2997,
    "start_hour": 14,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D2997",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_02_25,LEG_02_66"
  },
  {
    "id": 2998,
    "start_hour": 37,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D2998",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_03_14,LEG_03_176,LEG_03_240"
  },
  {
    "id": 2999,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2999",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_04_243,LEG_04_94,LEG_04_179"
  },
  {
    "id": 3000,
    "start_hour": 16,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D3000",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_02_2,LEG_02_62,LEG_02_68"
  },
  {
    "id": 3001,
    "start_hour": 52,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3001",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_03_21,LEG_03_255"
  },
  {
    "id": 3002,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3002",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_04_158,LEG_04_73,LEG_04_160"
  },
  {
    "id": 3003,
    "start_hour": 33,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3003",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_02_163,LEG_02_220"
  },
  {
    "id": 3004,
    "start_hour": 38,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3004",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_03_25,LEG_03_66"
  },
  {
    "id": 3005,
    "start_hour": 61,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3005",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_04_48,LEG_04_198,LEG_04_206,LEG_04_152"
  },
  {
    "id": 3006,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3006",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_02_4,LEG_02_135"
  },
  {
    "id": 3007,
    "start_hour": 38,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D3007",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_03_180,LEG_03_182"
  },
  {
    "id": 3008,
    "start_hour": 60,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3008",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_04_170,LEG_04_189"
  },
  {
    "id": 3009,
    "start_hour": 86,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D3009",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_05_94,LEG_05_151,LEG_05_64,LEG_05_147"
  },
  {
    "id": 3010,
    "start_hour": 35,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3010",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_02_108"
  },
  {
    "id": 3011,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3011",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_03_99,LEG_03_145"
  },
  {
    "id": 3012,
    "start_hour": 75,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3012",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_04_147,LEG_04_255"
  },
  {
    "id": 3013,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3013",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_05_145,LEG_05_3,LEG_05_122"
  },
  {
    "id": 3014,
    "start_hour": 33,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3014",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_02_94,LEG_02_179"
  },
  {
    "id": 3015,
    "start_hour": 38,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D3015",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_03_244,LEG_03_241"
  },
  {
    "id": 3016,
    "start_hour": 60,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3016",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_04_150,LEG_04_204,LEG_04_199,LEG_04_196"
  },
  {
    "id": 3017,
    "start_hour": 85,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D3017",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_05_40,LEG_05_179,LEG_05_181,LEG_05_93"
  },
  {
    "id": 3018,
    "start_hour": 196,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3018",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_09_231,LEG_09_138"
  },
  {
    "id": 3019,
    "start_hour": 204,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3019",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_10_9,LEG_10_153,LEG_10_142"
  },
  {
    "id": 3020,
    "start_hour": 245,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3020",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_11_210,LEG_11_128"
  },
  {
    "id": 3021,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3021",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_12_181,LEG_12_179,LEG_12_176"
  },
  {
    "id": 3022,
    "start_hour": 201,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3022",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_09_13,LEG_09_112"
  },
  {
    "id": 3023,
    "start_hour": 205,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3023",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_10_111,LEG_10_103"
  },
  {
    "id": 3024,
    "start_hour": 230,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D3024",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_11_25,LEG_11_66"
  },
  {
    "id": 3025,
    "start_hour": 253,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D3025",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_12_42,LEG_12_153,LEG_12_40,LEG_12_104"
  },
  {
    "id": 3026,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3026",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_09_83"
  },
  {
    "id": 3027,
    "start_hour": 218,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3027",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_10_71,LEG_10_151"
  },
  {
    "id": 3028,
    "start_hour": 240,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3028",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_11_160,LEG_11_175"
  },
  {
    "id": 3029,
    "start_hour": 264,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3029",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_12_161,LEG_12_127,LEG_12_204"
  },
  {
    "id": 3030,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3030",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_09_91"
  },
  {
    "id": 3031,
    "start_hour": 224,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3031",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_10_87,LEG_10_99"
  },
  {
    "id": 3032,
    "start_hour": 239,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3032",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_11_215,LEG_11_67"
  },
  {
    "id": 3033,
    "start_hour": 258,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3033",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_12_56"
  },
  {
    "id": 3034,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3034",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_09_223"
  },
  {
    "id": 3035,
    "start_hour": 218,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3035",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_10_179,LEG_10_96,LEG_10_215"
  },
  {
    "id": 3036,
    "start_hour": 228,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3036",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_11_88,LEG_11_50,LEG_11_31"
  },
  {
    "id": 3037,
    "start_hour": 252,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3037",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_12_8,LEG_12_144,LEG_12_213,LEG_12_41"
  },
  {
    "id": 3038,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3038",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_09_120,LEG_09_115"
  },
  {
    "id": 3039,
    "start_hour": 206,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3039",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_10_157,LEG_10_160,LEG_10_181"
  },
  {
    "id": 3040,
    "start_hour": 239,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3040",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_11_109,LEG_11_119,LEG_11_222"
  },
  {
    "id": 3041,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3041",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_09_55"
  },
  {
    "id": 3042,
    "start_hour": 222,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3042",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_10_3,LEG_10_22"
  },
  {
    "id": 3043,
    "start_hour": 244,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3043",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_11_64"
  },
  {
    "id": 3044,
    "start_hour": 191,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3044",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_09_81"
  },
  {
    "id": 3045,
    "start_hour": 213,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3045",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_10_124,LEG_10_244"
  },
  {
    "id": 3046,
    "start_hour": 234,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3046",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_11_56,LEG_11_231,LEG_11_118"
  },
  {
    "id": 3047,
    "start_hour": 203,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3047",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_09_197"
  },
  {
    "id": 3048,
    "start_hour": 220,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3048",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_10_189"
  },
  {
    "id": 3049,
    "start_hour": 203,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3049",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_09_38"
  },
  {
    "id": 3050,
    "start_hour": 220,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3050",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_10_43"
  },
  {
    "id": 3051,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3051",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_09_218,LEG_09_31,LEG_09_0"
  },
  {
    "id": 3052,
    "start_hour": 214,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3052",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_10_222,LEG_10_246,LEG_10_54"
  },
  {
    "id": 3053,
    "start_hour": 181,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3053",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_09_48,LEG_09_190"
  },
  {
    "id": 3054,
    "start_hour": 191,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3054",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_09_52,LEG_09_53"
  },
  {
    "id": 3055,
    "start_hour": 193,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3055",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_09_191"
  },
  {
    "id": 3056,
    "start_hour": 195,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3056",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_09_61,LEG_09_90,LEG_09_213,LEG_09_85"
  },
  {
    "id": 3057,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3057",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_31_41,LEG_31_48"
  },
  {
    "id": 3058,
    "start_hour": 709,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3058",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_31_49,LEG_31_52"
  },
  {
    "id": 3059,
    "start_hour": 722,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3059",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_31_242"
  },
  {
    "id": 3060,
    "start_hour": 722,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3060",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_31_47"
  },
  {
    "id": 3061,
    "start_hour": 720,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3061",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_31_208,LEG_31_266"
  },
  {
    "id": 3062,
    "start_hour": 720,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3062",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_31_107,LEG_31_109"
  },
  {
    "id": 3063,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3063",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_31_34,LEG_31_218"
  },
  {
    "id": 3064,
    "start_hour": 708,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3064",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_31_185,LEG_31_184"
  },
  {
    "id": 3065,
    "start_hour": 720,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3065",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_31_236,LEG_31_214,LEG_31_118,LEG_31_67"
  },
  {
    "id": 3066,
    "start_hour": 726,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3066",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_31_38,LEG_31_42"
  },
  {
    "id": 3067,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3067",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_08_227,LEG_08_24"
  },
  {
    "id": 3068,
    "start_hour": 182,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D3068",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_09_106,LEG_09_165,LEG_09_166"
  },
  {
    "id": 3069,
    "start_hour": 215,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3069",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_10_15,LEG_10_28"
  },
  {
    "id": 3070,
    "start_hour": 234,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3070",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_11_40,LEG_11_220,LEG_11_228"
  },
  {
    "id": 3071,
    "start_hour": 176,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3071",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_08_223"
  },
  {
    "id": 3072,
    "start_hour": 194,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3072",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_09_183,LEG_09_184"
  },
  {
    "id": 3073,
    "start_hour": 215,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3073",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_10_102,LEG_10_67"
  },
  {
    "id": 3074,
    "start_hour": 234,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3074",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_11_63"
  },
  {
    "id": 3075,
    "start_hour": 167,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3075",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_08_233,LEG_08_118,LEG_08_110"
  },
  {
    "id": 3076,
    "start_hour": 192,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3076",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_09_111,LEG_09_42"
  },
  {
    "id": 3077,
    "start_hour": 218,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3077",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_10_33,LEG_10_162"
  },
  {
    "id": 3078,
    "start_hour": 239,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3078",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_11_15,LEG_11_186"
  },
  {
    "id": 3079,
    "start_hour": 176,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3079",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_08_91"
  },
  {
    "id": 3080,
    "start_hour": 200,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D3080",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_09_89,LEG_09_232"
  },
  {
    "id": 3081,
    "start_hour": 216,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3081",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_10_235,LEG_10_11"
  },
  {
    "id": 3082,
    "start_hour": 236,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3082",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_11_10,LEG_11_184,LEG_11_84"
  },
  {
    "id": 3083,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3083",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_08_218,LEG_08_31,LEG_08_0"
  },
  {
    "id": 3084,
    "start_hour": 190,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3084",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_09_226,LEG_09_110"
  },
  {
    "id": 3085,
    "start_hour": 216,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3085",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_10_109,LEG_10_42"
  },
  {
    "id": 3086,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3086",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_11_33,LEG_11_153,LEG_11_69"
  },
  {
    "id": 3087,
    "start_hour": 167,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3087",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_08_81"
  },
  {
    "id": 3088,
    "start_hour": 189,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3088",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_09_126,LEG_09_248"
  },
  {
    "id": 3089,
    "start_hour": 210,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3089",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_10_56,LEG_10_229,LEG_10_116"
  },
  {
    "id": 3090,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3090",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_08_120,LEG_08_115"
  },
  {
    "id": 3091,
    "start_hour": 182,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D3091",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_09_161,LEG_09_164,LEG_09_22"
  },
  {
    "id": 3092,
    "start_hour": 220,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3092",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_10_64"
  },
  {
    "id": 3093,
    "start_hour": 177,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3093",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_08_13,LEG_08_112"
  },
  {
    "id": 3094,
    "start_hour": 181,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3094",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_09_113,LEG_09_107,LEG_09_185"
  },
  {
    "id": 3095,
    "start_hour": 215,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3095",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_10_107,LEG_10_117,LEG_10_220"
  },
  {
    "id": 3096,
    "start_hour": 169,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3096",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_08_191"
  },
  {
    "id": 3097,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3097",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_09_194"
  },
  {
    "id": 3098,
    "start_hour": 205,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3098",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_10_139,LEG_10_27,LEG_10_184,LEG_10_241,LEG_10_39"
  },
  {
    "id": 3099,
    "start_hour": 176,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3099",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_08_83"
  },
  {
    "id": 3100,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3100",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_09_123,LEG_09_199,LEG_09_196"
  },
  {
    "id": 3101,
    "start_hour": 179,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3101",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_08_197"
  },
  {
    "id": 3102,
    "start_hour": 196,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3102",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_09_193"
  },
  {
    "id": 3103,
    "start_hour": 179,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3103",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_08_38"
  },
  {
    "id": 3104,
    "start_hour": 196,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3104",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_09_43"
  },
  {
    "id": 3105,
    "start_hour": 157,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3105",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_08_48,LEG_08_190"
  },
  {
    "id": 3106,
    "start_hour": 172,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3106",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_08_231,LEG_08_229"
  },
  {
    "id": 3107,
    "start_hour": 167,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3107",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_08_52,LEG_08_53"
  },
  {
    "id": 3108,
    "start_hour": 171,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3108",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_08_61,LEG_08_90,LEG_08_213,LEG_08_85"
  },
  {
    "id": 3109,
    "start_hour": 98,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3109",
    "gerad_crew_id": "C0295",
    "flight_ids": "LEG_05_189,LEG_05_188"
  },
  {
    "id": 3110,
    "start_hour": 98,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3110",
    "gerad_crew_id": "C0296",
    "flight_ids": "LEG_05_231,LEG_05_234"
  },
  {
    "id": 3111,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3111",
    "gerad_crew_id": "C0297",
    "flight_ids": "LEG_05_111,LEG_05_108"
  },
  {
    "id": 3112,
    "start_hour": 106,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3112",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_05_233"
  },
  {
    "id": 3113,
    "start_hour": 109,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3113",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_06_40,LEG_06_42,LEG_06_73"
  },
  {
    "id": 3114,
    "start_hour": 146,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3114",
    "gerad_crew_id": "C0298",
    "flight_ids": "LEG_07_123"
  },
  {
    "id": 3115,
    "start_hour": 86,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D3115",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_05_109,LEG_05_68"
  },
  {
    "id": 3116,
    "start_hour": 125,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3116",
    "gerad_crew_id": "C0299",
    "flight_ids": "LEG_06_49,LEG_06_230"
  },
  {
    "id": 3117,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3117",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_05_186,LEG_05_138"
  },
  {
    "id": 3118,
    "start_hour": 110,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D3118",
    "gerad_crew_id": "C0300",
    "flight_ids": "LEG_06_96,LEG_06_149,LEG_06_16,LEG_06_184"
  },
  {
    "id": 3119,
    "start_hour": 107,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3119",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_05_116"
  },
  {
    "id": 3120,
    "start_hour": 126,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3120",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_06_113,LEG_06_114"
  },
  {
    "id": 3121,
    "start_hour": 146,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3121",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_07_202,LEG_07_206"
  },
  {
    "id": 3122,
    "start_hour": 170,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3122",
    "gerad_crew_id": "C0301",
    "flight_ids": "LEG_08_123"
  },
  {
    "id": 3123,
    "start_hour": 608,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3123",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_26_48"
  },
  {
    "id": 3124,
    "start_hour": 631,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3124",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_27_0,LEG_27_69"
  },
  {
    "id": 3125,
    "start_hour": 651,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D3125",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_28_85,LEG_28_171"
  },
  {
    "id": 3126,
    "start_hour": 673,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3126",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_29_174,LEG_29_229,LEG_29_223"
  },
  {
    "id": 3127,
    "start_hour": 612,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3127",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_26_62"
  },
  {
    "id": 3128,
    "start_hour": 629,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3128",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_27_15,LEG_27_139"
  },
  {
    "id": 3129,
    "start_hour": 649,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3129",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_28_173,LEG_28_142"
  },
  {
    "id": 3130,
    "start_hour": 673,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3130",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_29_212,LEG_29_221,LEG_29_79"
  },
  {
    "id": 3131,
    "start_hour": 589,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3131",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_26_40,LEG_26_169"
  },
  {
    "id": 3132,
    "start_hour": 601,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3132",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_26_170"
  },
  {
    "id": 3133,
    "start_hour": 601,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3133",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_26_61"
  },
  {
    "id": 3134,
    "start_hour": 618,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3134",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_27_45"
  },
  {
    "id": 3135,
    "start_hour": 611,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3135",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_26_176"
  },
  {
    "id": 3136,
    "start_hour": 629,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3136",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_27_152"
  },
  {
    "id": 3137,
    "start_hour": 609,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3137",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_26_198,LEG_26_99"
  },
  {
    "id": 3138,
    "start_hour": 613,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D3138",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_27_80,LEG_27_151"
  },
  {
    "id": 3139,
    "start_hour": 637,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3139",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_28_166"
  },
  {
    "id": 3140,
    "start_hour": 611,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3140",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_26_32"
  },
  {
    "id": 3141,
    "start_hour": 636,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3141",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_27_24"
  },
  {
    "id": 3142,
    "start_hour": 640,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D3142",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_28_176"
  },
  {
    "id": 3143,
    "start_hour": 599,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D3143",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_26_44,LEG_26_45,LEG_26_204"
  },
  {
    "id": 3144,
    "start_hour": 624,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3144",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_27_19,LEG_27_26"
  },
  {
    "id": 3145,
    "start_hour": 643,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3145",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_28_108,LEG_28_68,LEG_28_43"
  },
  {
    "id": 3146,
    "start_hour": 608,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3146",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_26_72"
  },
  {
    "id": 3147,
    "start_hour": 627,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3147",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_27_161,LEG_27_88"
  },
  {
    "id": 3148,
    "start_hour": 650,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3148",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_28_119,LEG_28_118,LEG_28_67"
  },
  {
    "id": 3149,
    "start_hour": 605,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3149",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_26_75"
  },
  {
    "id": 3150,
    "start_hour": 615,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D3150",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_27_122,LEG_27_52,LEG_27_75,LEG_27_101"
  },
  {
    "id": 3151,
    "start_hour": 637,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D3151",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_28_87,LEG_28_30,LEG_28_31"
  },
  {
    "id": 3152,
    "start_hour": 667,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3152",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_29_108,LEG_29_68,LEG_29_43"
  },
  {
    "id": 3153,
    "start_hour": 601,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3153",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_26_200,LEG_26_155,LEG_26_210"
  },
  {
    "id": 3154,
    "start_hour": 625,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3154",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_27_135,LEG_27_20"
  },
  {
    "id": 3155,
    "start_hour": 653,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3155",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_28_52,LEG_28_182"
  },
  {
    "id": 3156,
    "start_hour": 661,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D3156",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_29_167"
  },
  {
    "id": 3157,
    "start_hour": 608,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3157",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_26_79"
  },
  {
    "id": 3158,
    "start_hour": 633,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3158",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_27_59"
  },
  {
    "id": 3159,
    "start_hour": 639,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3159",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_28_238,LEG_28_62,LEG_28_148,LEG_28_203,LEG_28_244"
  },
  {
    "id": 3160,
    "start_hour": 265,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3160",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_12_171"
  },
  {
    "id": 3161,
    "start_hour": 272,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3161",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_12_174"
  },
  {
    "id": 3162,
    "start_hour": 263,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3162",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_12_46,LEG_12_47"
  },
  {
    "id": 3163,
    "start_hour": 265,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3163",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_12_74"
  },
  {
    "id": 3164,
    "start_hour": 275,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3164",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_12_35"
  },
  {
    "id": 3165,
    "start_hour": 292,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3165",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_13_36"
  },
  {
    "id": 3166,
    "start_hour": 272,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3166",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_12_48"
  },
  {
    "id": 3167,
    "start_hour": 294,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3167",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_13_3,LEG_13_102"
  },
  {
    "id": 3168,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3168",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_14_97,LEG_14_242,LEG_14_45"
  },
  {
    "id": 3169,
    "start_hour": 267,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3169",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_12_54,LEG_12_81,LEG_12_198"
  },
  {
    "id": 3170,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3170",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_13_168,LEG_13_169,LEG_13_5,LEG_13_0"
  },
  {
    "id": 3171,
    "start_hour": 310,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3171",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_14_225,LEG_14_251,LEG_14_53"
  },
  {
    "id": 3172,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3172",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_12_109,LEG_12_106"
  },
  {
    "id": 3173,
    "start_hour": 278,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3173",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_13_148,LEG_13_150,LEG_13_19"
  },
  {
    "id": 3174,
    "start_hour": 316,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3174",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_14_63"
  },
  {
    "id": 3175,
    "start_hour": 263,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3175",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_12_71"
  },
  {
    "id": 3176,
    "start_hour": 285,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3176",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_13_114,LEG_13_236"
  },
  {
    "id": 3177,
    "start_hour": 306,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3177",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_14_55,LEG_14_232,LEG_14_117"
  },
  {
    "id": 3178,
    "start_hour": 272,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3178",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_12_82"
  },
  {
    "id": 3179,
    "start_hour": 296,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3179",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_13_83"
  },
  {
    "id": 3180,
    "start_hour": 300,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D3180",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_14_10,LEG_14_154,LEG_14_77"
  },
  {
    "id": 3181,
    "start_hour": 332,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3181",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_15_70,LEG_15_182,LEG_15_83"
  },
  {
    "id": 3182,
    "start_hour": 276,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3182",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_12_61"
  },
  {
    "id": 3183,
    "start_hour": 292,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3183",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_13_18,LEG_13_172"
  },
  {
    "id": 3184,
    "start_hour": 311,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3184",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_14_108,LEG_14_118,LEG_14_223,LEG_14_189"
  },
  {
    "id": 3185,
    "start_hour": 337,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3185",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_15_40,LEG_15_244,LEG_15_38"
  },
  {
    "id": 3186,
    "start_hour": 273,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3186",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_12_14,LEG_12_101"
  },
  {
    "id": 3187,
    "start_hour": 277,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D3187",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_13_41,LEG_13_43,LEG_13_60"
  },
  {
    "id": 3188,
    "start_hour": 314,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3188",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_14_18,LEG_14_163"
  },
  {
    "id": 3189,
    "start_hour": 335,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3189",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_15_14,LEG_15_184"
  },
  {
    "id": 3190,
    "start_hour": 241,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3190",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_11_78,LEG_11_249"
  },
  {
    "id": 3191,
    "start_hour": 231,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3191",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_11_152,LEG_11_185"
  },
  {
    "id": 3192,
    "start_hour": 243,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3192",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_11_97,LEG_11_100"
  },
  {
    "id": 3193,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3193",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_11_16,LEG_11_23"
  },
  {
    "id": 3194,
    "start_hour": 231,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3194",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_11_36,LEG_11_219"
  },
  {
    "id": 3195,
    "start_hour": 240,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3195",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_11_251,LEG_11_252"
  },
  {
    "id": 3196,
    "start_hour": 241,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3196",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_11_171,LEG_11_172"
  },
  {
    "id": 3197,
    "start_hour": 228,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3197",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_11_74,LEG_11_70"
  },
  {
    "id": 3198,
    "start_hour": 232,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3198",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_11_116,LEG_11_12"
  },
  {
    "id": 3199,
    "start_hour": 230,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3199",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_11_242,LEG_11_239"
  },
  {
    "id": 3200,
    "start_hour": 239,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3200",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_11_255,LEG_11_60"
  },
  {
    "id": 3201,
    "start_hour": 260,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3201",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_12_77,LEG_12_119,LEG_12_126"
  },
  {
    "id": 3202,
    "start_hour": 244,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3202",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_11_77,LEG_11_121"
  },
  {
    "id": 3203,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3203",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_12_190,LEG_12_7"
  },
  {
    "id": 3204,
    "start_hour": 248,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3204",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_11_250,LEG_11_1"
  },
  {
    "id": 3205,
    "start_hour": 247,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3205",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_11_147,LEG_11_151"
  },
  {
    "id": 3206,
    "start_hour": 244,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3206",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_11_93,LEG_11_92"
  },
  {
    "id": 3207,
    "start_hour": 245,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3207",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_11_19,LEG_11_26"
  },
  {
    "id": 3208,
    "start_hour": 247,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3208",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_11_114,LEG_11_117"
  },
  {
    "id": 3209,
    "start_hour": 230,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D3209",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_11_34,LEG_11_37"
  },
  {
    "id": 3210,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3210",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_11_157,LEG_11_103"
  },
  {
    "id": 3211,
    "start_hour": 255,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D3211",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_12_125,LEG_12_202,LEG_12_197,LEG_12_182"
  },
  {
    "id": 3212,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3212",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_13_198,LEG_13_8"
  },
  {
    "id": 3213,
    "start_hour": 245,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3213",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_11_154,LEG_11_149,LEG_11_230"
  },
  {
    "id": 3214,
    "start_hour": 264,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3214",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_12_210,LEG_12_211"
  },
  {
    "id": 3215,
    "start_hour": 290,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3215",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_13_231,LEG_13_152"
  },
  {
    "id": 3216,
    "start_hour": 311,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3216",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_14_16,LEG_14_218,LEG_14_217"
  },
  {
    "id": 3217,
    "start_hour": 249,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3217",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_11_161,LEG_11_218"
  },
  {
    "id": 3218,
    "start_hour": 254,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3218",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_12_208"
  },
  {
    "id": 3219,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3219",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_13_220,LEG_13_92,LEG_13_203,LEG_13_166"
  },
  {
    "id": 3220,
    "start_hour": 311,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3220",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_14_250,LEG_14_109,LEG_14_106"
  },
  {
    "id": 3221,
    "start_hour": 243,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3221",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_11_238"
  },
  {
    "id": 3222,
    "start_hour": 266,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3222",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_12_214,LEG_12_166"
  },
  {
    "id": 3223,
    "start_hour": 276,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3223",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_13_82,LEG_13_226,LEG_13_108,LEG_13_211,LEG_13_37"
  },
  {
    "id": 3224,
    "start_hour": 311,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3224",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_14_213,LEG_14_224,LEG_14_141"
  },
  {
    "id": 3225,
    "start_hour": 251,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3225",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_11_7"
  },
  {
    "id": 3226,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3226",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_12_4,LEG_12_131"
  },
  {
    "id": 3227,
    "start_hour": 291,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3227",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_13_133,LEG_13_120"
  },
  {
    "id": 3228,
    "start_hour": 301,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3228",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_14_130"
  },
  {
    "id": 3229,
    "start_hour": 245,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3229",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_11_166,LEG_11_167,LEG_11_82"
  },
  {
    "id": 3230,
    "start_hour": 267,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3230",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_12_178,LEG_12_180"
  },
  {
    "id": 3231,
    "start_hour": 288,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3231",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_13_104,LEG_13_35"
  },
  {
    "id": 3232,
    "start_hour": 314,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3232",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_14_34"
  },
  {
    "id": 3233,
    "start_hour": 251,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3233",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_11_108"
  },
  {
    "id": 3234,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3234",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_12_90,LEG_12_105"
  },
  {
    "id": 3235,
    "start_hour": 286,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3235",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_13_215,LEG_13_237,LEG_13_47"
  },
  {
    "id": 3236,
    "start_hour": 312,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3236",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_14_159,LEG_14_129,LEG_14_132"
  },
  {
    "id": 3237,
    "start_hour": 232,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D3237",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_11_2,LEG_11_62,LEG_11_38"
  },
  {
    "id": 3238,
    "start_hour": 268,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3238",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_12_39,LEG_12_203"
  },
  {
    "id": 3239,
    "start_hour": 291,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3239",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_13_124,LEG_13_106,LEG_13_107"
  },
  {
    "id": 3240,
    "start_hour": 249,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3240",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_11_94,LEG_11_177"
  },
  {
    "id": 3241,
    "start_hour": 254,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D3241",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_12_24,LEG_12_59"
  },
  {
    "id": 3242,
    "start_hour": 277,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3242",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_13_40,LEG_13_186,LEG_13_189,LEG_13_99"
  },
  {
    "id": 3243,
    "start_hour": 251,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3243",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_11_22"
  },
  {
    "id": 3244,
    "start_hour": 268,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3244",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_12_58,LEG_12_172"
  },
  {
    "id": 3245,
    "start_hour": 289,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3245",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_13_34,LEG_13_212,LEG_13_51,LEG_13_39"
  },
  {
    "id": 3246,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3246",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_14_164,LEG_14_5,LEG_14_133"
  },
  {
    "id": 3247,
    "start_hour": 240,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3247",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_11_130,LEG_11_137,LEG_11_144"
  },
  {
    "id": 3248,
    "start_hour": 269,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3248",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_12_191,LEG_12_117"
  },
  {
    "id": 3249,
    "start_hour": 290,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3249",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_13_190,LEG_13_170,LEG_13_242"
  },
  {
    "id": 3250,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3250",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_14_155,LEG_14_73,LEG_14_157"
  },
  {
    "id": 3251,
    "start_hour": 251,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3251",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_11_32"
  },
  {
    "id": 3252,
    "start_hour": 275,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3252",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_12_177"
  },
  {
    "id": 3253,
    "start_hour": 292,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3253",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_13_181,LEG_13_180"
  },
  {
    "id": 3254,
    "start_hour": 313,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3254",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_14_41,LEG_14_119,LEG_14_114"
  },
  {
    "id": 3255,
    "start_hour": 634,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3255",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_27_63"
  },
  {
    "id": 3256,
    "start_hour": 636,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3256",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_28_239,LEG_28_174,LEG_28_170,LEG_28_60"
  },
  {
    "id": 3257,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3257",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_29_197,LEG_29_190"
  },
  {
    "id": 3258,
    "start_hour": 697,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3258",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_30_94"
  },
  {
    "id": 3259,
    "start_hour": 634,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3259",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_27_207,LEG_27_216"
  },
  {
    "id": 3260,
    "start_hour": 639,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3260",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_28_162,LEG_28_157"
  },
  {
    "id": 3261,
    "start_hour": 661,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D3261",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_29_137,LEG_29_249,LEG_29_63"
  },
  {
    "id": 3262,
    "start_hour": 693,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3262",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_30_139,LEG_30_234,LEG_30_152"
  },
  {
    "id": 3263,
    "start_hour": 629,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3263",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_27_9,LEG_27_165"
  },
  {
    "id": 3264,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3264",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_28_252,LEG_28_0"
  },
  {
    "id": 3265,
    "start_hour": 661,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3265",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_29_71,LEG_29_103,LEG_29_215,LEG_29_145"
  },
  {
    "id": 3266,
    "start_hour": 612,
    "duration_hours": 29,
    "required_skill": "A321",
    "gerad_duty_id": "D3266",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_27_121,LEG_27_167,LEG_27_166"
  },
  {
    "id": 3267,
    "start_hour": 654,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3267",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_28_204,LEG_28_14"
  },
  {
    "id": 3268,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3268",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_29_162,LEG_29_259,LEG_29_126"
  },
  {
    "id": 3269,
    "start_hour": 629,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3269",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_27_53,LEG_27_10"
  },
  {
    "id": 3270,
    "start_hour": 651,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3270",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_28_186,LEG_28_105,LEG_28_223"
  },
  {
    "id": 3271,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3271",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_29_37,LEG_29_33"
  },
  {
    "id": 3272,
    "start_hour": 699,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3272",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_30_144"
  },
  {
    "id": 3273,
    "start_hour": 625,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3273",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_27_87,LEG_27_60"
  },
  {
    "id": 3274,
    "start_hour": 625,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3274",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_27_223,LEG_27_148"
  },
  {
    "id": 3275,
    "start_hour": 628,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3275",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_27_66,LEG_27_70"
  },
  {
    "id": 3276,
    "start_hour": 626,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3276",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_27_111,LEG_27_112"
  },
  {
    "id": 3277,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3277",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_27_5,LEG_27_4"
  },
  {
    "id": 3278,
    "start_hour": 625,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3278",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_27_218,LEG_27_219"
  },
  {
    "id": 3279,
    "start_hour": 632,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3279",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_27_78"
  },
  {
    "id": 3280,
    "start_hour": 633,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D3280",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_27_1"
  },
  {
    "id": 3281,
    "start_hour": 637,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D3281",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_28_1"
  },
  {
    "id": 3282,
    "start_hour": 633,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3282",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_27_50"
  },
  {
    "id": 3283,
    "start_hour": 638,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3283",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_28_59"
  },
  {
    "id": 3284,
    "start_hour": 633,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3284",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_27_147"
  },
  {
    "id": 3285,
    "start_hour": 637,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3285",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_28_179"
  },
  {
    "id": 3286,
    "start_hour": 632,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D3286",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_27_119"
  },
  {
    "id": 3287,
    "start_hour": 636,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3287",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_28_149"
  },
  {
    "id": 3288,
    "start_hour": 633,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3288",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_27_61"
  },
  {
    "id": 3289,
    "start_hour": 638,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3289",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_28_71"
  },
  {
    "id": 3290,
    "start_hour": 630,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3290",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_27_85,LEG_27_86"
  },
  {
    "id": 3291,
    "start_hour": 630,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3291",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_27_124,LEG_27_125"
  },
  {
    "id": 3292,
    "start_hour": 630,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3292",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_27_17,LEG_27_18"
  },
  {
    "id": 3293,
    "start_hour": 625,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D3293",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_27_220,LEG_27_72,LEG_27_158,LEG_27_157"
  },
  {
    "id": 3294,
    "start_hour": 649,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3294",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_28_94,LEG_28_63"
  },
  {
    "id": 3295,
    "start_hour": 669,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3295",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_29_139,LEG_29_235,LEG_29_153"
  },
  {
    "id": 3296,
    "start_hour": 614,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3296",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_27_208"
  },
  {
    "id": 3297,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3297",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_28_151,LEG_28_57"
  },
  {
    "id": 3298,
    "start_hour": 672,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3298",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_29_24,LEG_29_134,LEG_29_131"
  },
  {
    "id": 3299,
    "start_hour": 625,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3299",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_27_103,LEG_27_180,LEG_27_179"
  },
  {
    "id": 3300,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3300",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_28_37,LEG_28_33"
  },
  {
    "id": 3301,
    "start_hour": 675,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3301",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_29_144"
  },
  {
    "id": 3302,
    "start_hour": 614,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3302",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_27_23,LEG_27_44"
  },
  {
    "id": 3303,
    "start_hour": 638,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3303",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_28_39,LEG_28_35,LEG_28_66"
  },
  {
    "id": 3304,
    "start_hour": 675,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3304",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_29_56"
  },
  {
    "id": 3305,
    "start_hour": 636,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3305",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_27_74"
  },
  {
    "id": 3306,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3306",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_28_29"
  },
  {
    "id": 3307,
    "start_hour": 628,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3307",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_27_107,LEG_27_109"
  },
  {
    "id": 3308,
    "start_hour": 637,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3308",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_28_70,LEG_28_103,LEG_28_214,LEG_28_144"
  },
  {
    "id": 3309,
    "start_hour": 612,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D3309",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_27_48,LEG_27_49"
  },
  {
    "id": 3310,
    "start_hour": 637,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3310",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_28_100,LEG_28_187,LEG_28_197,LEG_28_147"
  },
  {
    "id": 3311,
    "start_hour": 629,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3311",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_27_134,LEG_27_133"
  },
  {
    "id": 3312,
    "start_hour": 639,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3312",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_28_98,LEG_28_265,LEG_28_165,LEG_28_163"
  },
  {
    "id": 3313,
    "start_hour": 628,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3313",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_27_136,LEG_27_143"
  },
  {
    "id": 3314,
    "start_hour": 637,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3314",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_28_126,LEG_28_164,LEG_28_110,LEG_28_233"
  },
  {
    "id": 3315,
    "start_hour": 631,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3315",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_27_67,LEG_27_181"
  },
  {
    "id": 3316,
    "start_hour": 639,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3316",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_28_225"
  },
  {
    "id": 3317,
    "start_hour": 674,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3317",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_29_228,LEG_29_110"
  },
  {
    "id": 3318,
    "start_hour": 696,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3318",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_30_24,LEG_30_134,LEG_30_131"
  },
  {
    "id": 3319,
    "start_hour": 146,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3319",
    "gerad_crew_id": "C0302",
    "flight_ids": "LEG_07_210,LEG_07_209"
  },
  {
    "id": 3320,
    "start_hour": 146,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3320",
    "gerad_crew_id": "C0303",
    "flight_ids": "LEG_07_257,LEG_07_260"
  },
  {
    "id": 3321,
    "start_hour": 134,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D3321",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_07_122,LEG_07_79"
  },
  {
    "id": 3322,
    "start_hour": 173,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3322",
    "gerad_crew_id": "C0304",
    "flight_ids": "LEG_08_57,LEG_08_247"
  },
  {
    "id": 3323,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3323",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_07_207,LEG_07_152"
  },
  {
    "id": 3324,
    "start_hour": 158,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D3324",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_08_180,LEG_08_182"
  },
  {
    "id": 3325,
    "start_hour": 180,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D3325",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_09_150,LEG_09_204,LEG_09_205"
  },
  {
    "id": 3326,
    "start_hour": 218,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3326",
    "gerad_crew_id": "C0305",
    "flight_ids": "LEG_10_121"
  },
  {
    "id": 3327,
    "start_hour": 146,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3327",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_07_212"
  },
  {
    "id": 3328,
    "start_hour": 156,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D3328",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_08_88,LEG_08_246,LEG_08_172,LEG_08_80,LEG_08_232"
  },
  {
    "id": 3329,
    "start_hour": 192,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3329",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_09_239,LEG_09_177"
  },
  {
    "id": 3330,
    "start_hour": 216,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3330",
    "gerad_crew_id": "C0255",
    "flight_ids": "LEG_10_174,LEG_10_144,LEG_10_203"
  },
  {
    "id": 3331,
    "start_hour": 134,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3331",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_07_59,LEG_07_96"
  },
  {
    "id": 3332,
    "start_hour": 158,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3332",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_08_235"
  },
  {
    "id": 3333,
    "start_hour": 194,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3333",
    "gerad_crew_id": "C0256",
    "flight_ids": "LEG_09_234,LEG_09_20,LEG_09_200"
  },
  {
    "id": 3334,
    "start_hour": 155,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3334",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_07_129"
  },
  {
    "id": 3335,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3335",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_08_127,LEG_08_238"
  },
  {
    "id": 3336,
    "start_hour": 181,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D3336",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_09_142,LEG_09_27,LEG_09_240"
  },
  {
    "id": 3337,
    "start_hour": 218,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3337",
    "gerad_crew_id": "C0257",
    "flight_ids": "LEG_10_239,LEG_10_180,LEG_10_171,LEG_10_199"
  },
  {
    "id": 3338,
    "start_hour": 74,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3338",
    "gerad_crew_id": "C0258",
    "flight_ids": "LEG_04_256,LEG_04_259"
  },
  {
    "id": 3339,
    "start_hour": 74,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3339",
    "gerad_crew_id": "C0259",
    "flight_ids": "LEG_04_209,LEG_04_208"
  },
  {
    "id": 3340,
    "start_hour": 62,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3340",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_04_59,LEG_04_190"
  },
  {
    "id": 3341,
    "start_hour": 85,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D3341",
    "gerad_crew_id": "C0260",
    "flight_ids": "LEG_05_176,LEG_05_112"
  },
  {
    "id": 3342,
    "start_hour": 62,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3342",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_04_122,LEG_04_79"
  },
  {
    "id": 3343,
    "start_hour": 101,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3343",
    "gerad_crew_id": "C0261",
    "flight_ids": "LEG_05_49,LEG_05_223"
  },
  {
    "id": 3344,
    "start_hour": 82,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3344",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_04_258"
  },
  {
    "id": 3345,
    "start_hour": 85,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D3345",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_05_41,LEG_05_43,LEG_05_42,LEG_05_28"
  },
  {
    "id": 3346,
    "start_hour": 108,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D3346",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_06_6,LEG_06_159,LEG_06_223"
  },
  {
    "id": 3347,
    "start_hour": 146,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D3347",
    "gerad_crew_id": "C0262",
    "flight_ids": "LEG_07_244,LEG_07_184,LEG_07_175,LEG_07_204"
  },
  {
    "id": 3348,
    "start_hour": 82,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3348",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_04_238"
  },
  {
    "id": 3349,
    "start_hour": 85,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3349",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_05_101,LEG_05_95,LEG_05_66"
  },
  {
    "id": 3350,
    "start_hour": 124,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3350",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_06_74,LEG_06_81"
  },
  {
    "id": 3351,
    "start_hour": 153,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3351",
    "gerad_crew_id": "C0263",
    "flight_ids": "LEG_07_58"
  },
  {
    "id": 3352,
    "start_hour": 723,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3352",
    "gerad_crew_id": "C0264",
    "flight_ids": "LEG_31_195,LEG_31_196"
  },
  {
    "id": 3353,
    "start_hour": 712,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3353",
    "gerad_crew_id": "C0265",
    "flight_ids": "LEG_31_192,LEG_31_12"
  },
  {
    "id": 3354,
    "start_hour": 712,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3354",
    "gerad_crew_id": "C0266",
    "flight_ids": "LEG_31_16,LEG_31_246"
  },
  {
    "id": 3355,
    "start_hour": 711,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3355",
    "gerad_crew_id": "C0267",
    "flight_ids": "LEG_31_45,LEG_31_206"
  },
  {
    "id": 3356,
    "start_hour": 723,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3356",
    "gerad_crew_id": "C0268",
    "flight_ids": "LEG_31_247,LEG_31_244"
  },
  {
    "id": 3357,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3357",
    "gerad_crew_id": "C0269",
    "flight_ids": "LEG_31_255,LEG_31_258"
  },
  {
    "id": 3358,
    "start_hour": 723,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3358",
    "gerad_crew_id": "C0270",
    "flight_ids": "LEG_31_116,LEG_31_115"
  },
  {
    "id": 3359,
    "start_hour": 723,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3359",
    "gerad_crew_id": "C0271",
    "flight_ids": "LEG_31_207"
  },
  {
    "id": 3360,
    "start_hour": 723,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3360",
    "gerad_crew_id": "C0272",
    "flight_ids": "LEG_31_199"
  },
  {
    "id": 3361,
    "start_hour": 248,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3361",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_11_91"
  },
  {
    "id": 3362,
    "start_hour": 273,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3362",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_12_51"
  },
  {
    "id": 3363,
    "start_hour": 278,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3363",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_13_110,LEG_13_70"
  },
  {
    "id": 3364,
    "start_hour": 300,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3364",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_14_87,LEG_14_246,LEG_14_245,LEG_14_39"
  },
  {
    "id": 3365,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3365",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_11_225,LEG_11_24"
  },
  {
    "id": 3366,
    "start_hour": 254,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D3366",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_12_95,LEG_12_151,LEG_12_152"
  },
  {
    "id": 3367,
    "start_hour": 287,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3367",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_13_13,LEG_13_11"
  },
  {
    "id": 3368,
    "start_hour": 308,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3368",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_14_11,LEG_14_183,LEG_14_84"
  },
  {
    "id": 3369,
    "start_hour": 249,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3369",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_11_13,LEG_11_112"
  },
  {
    "id": 3370,
    "start_hour": 253,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D3370",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_12_102,LEG_12_96"
  },
  {
    "id": 3371,
    "start_hour": 276,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D3371",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_13_7,LEG_13_161,LEG_13_228"
  },
  {
    "id": 3372,
    "start_hour": 314,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D3372",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_14_243,LEG_14_185"
  },
  {
    "id": 3373,
    "start_hour": 248,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3373",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_11_83"
  },
  {
    "id": 3374,
    "start_hour": 266,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3374",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_12_64,LEG_12_141,LEG_12_62"
  },
  {
    "id": 3375,
    "start_hour": 277,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D3375",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_13_128,LEG_13_23,LEG_13_24"
  },
  {
    "id": 3376,
    "start_hour": 306,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3376",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_14_40,LEG_14_221,LEG_14_229"
  },
  {
    "id": 3377,
    "start_hour": 246,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3377",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_11_170,LEG_11_80"
  },
  {
    "id": 3378,
    "start_hour": 252,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D3378",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_12_28,LEG_12_142,LEG_12_10"
  },
  {
    "id": 3379,
    "start_hour": 284,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3379",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_13_10,LEG_13_174"
  },
  {
    "id": 3380,
    "start_hour": 239,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3380",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_11_81"
  },
  {
    "id": 3381,
    "start_hour": 261,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3381",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_12_115,LEG_12_220"
  },
  {
    "id": 3382,
    "start_hour": 282,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3382",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_13_49,LEG_13_209,LEG_13_217"
  },
  {
    "id": 3383,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3383",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_11_120,LEG_11_115"
  },
  {
    "id": 3384,
    "start_hour": 254,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D3384",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_12_148,LEG_12_150,LEG_12_97"
  },
  {
    "id": 3385,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3385",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_13_93,LEG_13_230,LEG_13_38"
  },
  {
    "id": 3386,
    "start_hour": 248,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3386",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_11_55"
  },
  {
    "id": 3387,
    "start_hour": 270,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3387",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_12_2,LEG_12_22"
  },
  {
    "id": 3388,
    "start_hour": 292,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3388",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_13_59"
  },
  {
    "id": 3389,
    "start_hour": 249,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3389",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_11_44,LEG_11_35"
  },
  {
    "id": 3390,
    "start_hour": 254,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3390",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_12_215,LEG_12_212"
  },
  {
    "id": 3391,
    "start_hour": 276,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3391",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_13_9,LEG_13_144,LEG_13_142,LEG_13_64"
  },
  {
    "id": 3392,
    "start_hour": 251,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3392",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_11_195"
  },
  {
    "id": 3393,
    "start_hour": 268,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3393",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_12_173"
  },
  {
    "id": 3394,
    "start_hour": 241,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3394",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_11_216,LEG_11_244,LEG_11_243,LEG_11_39"
  },
  {
    "id": 3395,
    "start_hour": 241,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3395",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_11_189"
  },
  {
    "id": 3396,
    "start_hour": 241,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3396",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_11_213,LEG_11_212"
  },
  {
    "id": 3397,
    "start_hour": 229,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3397",
    "gerad_crew_id": "C0224",
    "flight_ids": "LEG_11_48,LEG_11_188"
  },
  {
    "id": 3398,
    "start_hour": 244,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3398",
    "gerad_crew_id": "C0225",
    "flight_ids": "LEG_11_229,LEG_11_227"
  },
  {
    "id": 3399,
    "start_hour": 239,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3399",
    "gerad_crew_id": "C0226",
    "flight_ids": "LEG_11_52,LEG_11_53"
  },
  {
    "id": 3400,
    "start_hour": 243,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3400",
    "gerad_crew_id": "C0227",
    "flight_ids": "LEG_11_61,LEG_11_90,LEG_11_211,LEG_11_85"
  },
  {
    "id": 3401,
    "start_hour": 706,
    "duration_hours": 2,
    "required_skill": "A319",
    "gerad_duty_id": "D3401",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_30_208"
  },
  {
    "id": 3402,
    "start_hour": 708,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3402",
    "gerad_crew_id": "C0228",
    "flight_ids": "LEG_31_237,LEG_31_39,LEG_31_35,LEG_31_221,LEG_31_79"
  },
  {
    "id": 3403,
    "start_hour": 706,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3403",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_30_104"
  },
  {
    "id": 3404,
    "start_hour": 708,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D3404",
    "gerad_crew_id": "C0229",
    "flight_ids": "LEG_31_40,LEG_31_213,LEG_31_210"
  },
  {
    "id": 3405,
    "start_hour": 684,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D3405",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_30_184,LEG_30_183,LEG_30_182"
  },
  {
    "id": 3406,
    "start_hour": 709,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3406",
    "gerad_crew_id": "C0230",
    "flight_ids": "LEG_31_167"
  },
  {
    "id": 3407,
    "start_hour": 701,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3407",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_30_117,LEG_30_256"
  },
  {
    "id": 3408,
    "start_hour": 727,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3408",
    "gerad_crew_id": "C0231",
    "flight_ids": "LEG_31_15,LEG_31_55"
  },
  {
    "id": 3409,
    "start_hour": 701,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3409",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_30_210,LEG_30_125"
  },
  {
    "id": 3410,
    "start_hour": 709,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3410",
    "gerad_crew_id": "C0232",
    "flight_ids": "LEG_31_5,LEG_31_191,LEG_31_245"
  },
  {
    "id": 3411,
    "start_hour": 698,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3411",
    "gerad_crew_id": "C0233",
    "flight_ids": "LEG_30_218,LEG_30_224"
  },
  {
    "id": 3412,
    "start_hour": 684,
    "duration_hours": 27,
    "required_skill": "A321",
    "gerad_duty_id": "D3412",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_30_78,LEG_30_96,LEG_30_171"
  },
  {
    "id": 3413,
    "start_hour": 721,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3413",
    "gerad_crew_id": "C0234",
    "flight_ids": "LEG_31_174,LEG_31_229,LEG_31_223"
  },
  {
    "id": 3414,
    "start_hour": 685,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D3414",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_30_49,LEG_30_20,LEG_30_4"
  },
  {
    "id": 3415,
    "start_hour": 725,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3415",
    "gerad_crew_id": "C0235",
    "flight_ids": "LEG_31_77"
  },
  {
    "id": 3416,
    "start_hour": 696,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3416",
    "gerad_crew_id": "C0236",
    "flight_ids": "LEG_30_207,LEG_30_265"
  },
  {
    "id": 3417,
    "start_hour": 696,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D3417",
    "gerad_crew_id": "C0237",
    "flight_ids": "LEG_30_107,LEG_30_109"
  },
  {
    "id": 3418,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3418",
    "gerad_crew_id": "C0238",
    "flight_ids": "LEG_30_34,LEG_30_217"
  },
  {
    "id": 3419,
    "start_hour": 698,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3419",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_30_241"
  },
  {
    "id": 3420,
    "start_hour": 728,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3420",
    "gerad_crew_id": "C0239",
    "flight_ids": "LEG_31_243"
  },
  {
    "id": 3421,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3421",
    "gerad_crew_id": "C0240",
    "flight_ids": "LEG_30_41,LEG_30_48"
  },
  {
    "id": 3422,
    "start_hour": 696,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3422",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_30_64"
  },
  {
    "id": 3423,
    "start_hour": 727,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3423",
    "gerad_crew_id": "C0241",
    "flight_ids": "LEG_31_189"
  },
  {
    "id": 3424,
    "start_hour": 698,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D3424",
    "gerad_crew_id": "C0242",
    "flight_ids": "LEG_30_47"
  },
  {
    "id": 3425,
    "start_hour": 696,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3425",
    "gerad_crew_id": "C0243",
    "flight_ids": "LEG_30_216,LEG_30_215,LEG_30_220,LEG_30_79"
  },
  {
    "id": 3426,
    "start_hour": 702,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3426",
    "gerad_crew_id": "C0244",
    "flight_ids": "LEG_30_38,LEG_30_42"
  },
  {
    "id": 3427,
    "start_hour": 409,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D3427",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_18_78,LEG_18_249"
  },
  {
    "id": 3428,
    "start_hour": 400,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3428",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_18_115,LEG_18_12"
  },
  {
    "id": 3429,
    "start_hour": 408,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3429",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_18_138,LEG_18_139"
  },
  {
    "id": 3430,
    "start_hour": 411,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3430",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_18_96,LEG_18_99"
  },
  {
    "id": 3431,
    "start_hour": 396,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3431",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_18_74,LEG_18_70"
  },
  {
    "id": 3432,
    "start_hour": 399,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3432",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_18_35,LEG_18_216"
  },
  {
    "id": 3433,
    "start_hour": 408,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3433",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_18_251,LEG_18_252"
  },
  {
    "id": 3434,
    "start_hour": 399,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3434",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_18_150,LEG_18_183"
  },
  {
    "id": 3435,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3435",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_18_15,LEG_18_22"
  },
  {
    "id": 3436,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3436",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_18_169,LEG_18_220,LEG_18_63"
  },
  {
    "id": 3437,
    "start_hour": 430,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3437",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_19_200,LEG_19_201,LEG_19_130"
  },
  {
    "id": 3438,
    "start_hour": 400,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3438",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_18_2,LEG_18_61,LEG_18_37"
  },
  {
    "id": 3439,
    "start_hour": 434,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3439",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_19_5"
  },
  {
    "id": 3440,
    "start_hour": 412,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3440",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_18_77,LEG_18_120"
  },
  {
    "id": 3441,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3441",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_19_189,LEG_19_8"
  },
  {
    "id": 3442,
    "start_hour": 407,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3442",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_18_255,LEG_18_59"
  },
  {
    "id": 3443,
    "start_hour": 428,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3443",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_19_72,LEG_19_120,LEG_19_125"
  },
  {
    "id": 3444,
    "start_hour": 413,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3444",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_18_18,LEG_18_25"
  },
  {
    "id": 3445,
    "start_hour": 415,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3445",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_18_145,LEG_18_149"
  },
  {
    "id": 3446,
    "start_hour": 416,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3446",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_18_250,LEG_18_1"
  },
  {
    "id": 3447,
    "start_hour": 412,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3447",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_18_92,LEG_18_91"
  },
  {
    "id": 3448,
    "start_hour": 415,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3448",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_18_113,LEG_18_116"
  },
  {
    "id": 3449,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3449",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_18_155,LEG_18_102"
  },
  {
    "id": 3450,
    "start_hour": 413,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D3450",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_18_164,LEG_18_165"
  },
  {
    "id": 3451,
    "start_hour": 420,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D3451",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_19_26,LEG_19_142,LEG_19_11"
  },
  {
    "id": 3452,
    "start_hour": 452,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3452",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_20_9,LEG_20_67,LEG_20_231"
  },
  {
    "id": 3453,
    "start_hour": 398,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D3453",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_18_33,LEG_18_29,LEG_18_75"
  },
  {
    "id": 3454,
    "start_hour": 436,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3454",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_19_71,LEG_19_47"
  },
  {
    "id": 3455,
    "start_hour": 462,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D3455",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_20_2"
  },
  {
    "id": 3456,
    "start_hour": 396,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3456",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_18_146,LEG_18_200,LEG_18_201"
  },
  {
    "id": 3457,
    "start_hour": 434,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3457",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_19_63,LEG_19_166,LEG_19_181"
  },
  {
    "id": 3458,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3458",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_20_114,LEG_20_65"
  },
  {
    "id": 3459,
    "start_hour": 417,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3459",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_18_159,LEG_18_214"
  },
  {
    "id": 3460,
    "start_hour": 422,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D3460",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_19_148,LEG_19_150"
  },
  {
    "id": 3461,
    "start_hour": 444,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3461",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_20_154"
  },
  {
    "id": 3462,
    "start_hour": 478,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3462",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_21_214,LEG_21_228,LEG_21_142"
  },
  {
    "id": 3463,
    "start_hour": 413,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3463",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_18_152,LEG_18_147,LEG_18_231"
  },
  {
    "id": 3464,
    "start_hour": 432,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3464",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_19_210,LEG_19_160"
  },
  {
    "id": 3465,
    "start_hour": 456,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3465",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_20_160,LEG_20_10"
  },
  {
    "id": 3466,
    "start_hour": 476,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3466",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_21_10,LEG_21_72,LEG_21_139"
  },
  {
    "id": 3467,
    "start_hour": 419,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3467",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_18_7"
  },
  {
    "id": 3468,
    "start_hour": 436,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3468",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_19_37,LEG_19_171"
  },
  {
    "id": 3469,
    "start_hour": 457,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3469",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_20_32,LEG_20_33"
  },
  {
    "id": 3470,
    "start_hour": 482,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3470",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_21_32"
  },
  {
    "id": 3471,
    "start_hour": 410,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3471",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_18_184,LEG_18_86"
  },
  {
    "id": 3472,
    "start_hour": 441,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3472",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_19_50"
  },
  {
    "id": 3473,
    "start_hour": 447,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3473",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_20_191,LEG_20_4,LEG_20_121"
  },
  {
    "id": 3474,
    "start_hour": 469,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3474",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_21_133"
  },
  {
    "id": 3475,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3475",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_18_4,LEG_18_215"
  },
  {
    "id": 3476,
    "start_hour": 422,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D3476",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_19_97,LEG_19_151,LEG_19_88,LEG_19_193"
  },
  {
    "id": 3477,
    "start_hour": 444,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3477",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_20_80,LEG_20_220,LEG_20_109,LEG_20_206,LEG_20_35"
  },
  {
    "id": 3478,
    "start_hour": 479,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3478",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_21_213,LEG_21_223,LEG_21_172"
  },
  {
    "id": 3479,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3479",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_18_182,LEG_18_83,LEG_18_188"
  },
  {
    "id": 3480,
    "start_hour": 433,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3480",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_19_36,LEG_19_73"
  },
  {
    "id": 3481,
    "start_hour": 464,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3481",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_20_81,LEG_20_70"
  },
  {
    "id": 3482,
    "start_hour": 483,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3482",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_21_195,LEG_21_203,LEG_21_150"
  },
  {
    "id": 3483,
    "start_hour": 419,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3483",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_18_107"
  },
  {
    "id": 3484,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3484",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_19_89,LEG_19_131"
  },
  {
    "id": 3485,
    "start_hour": 459,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3485",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_20_132,LEG_20_235"
  },
  {
    "id": 3486,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3486",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_21_156,LEG_21_74,LEG_21_158"
  },
  {
    "id": 3487,
    "start_hour": 419,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D3487",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_18_31"
  },
  {
    "id": 3488,
    "start_hour": 434,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3488",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_19_30,LEG_19_132"
  },
  {
    "id": 3489,
    "start_hour": 461,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3489",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_20_193,LEG_20_117"
  },
  {
    "id": 3490,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3490",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_21_199,LEG_21_198,LEG_21_106"
  },
  {
    "id": 3491,
    "start_hour": 672,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3491",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_29_64"
  },
  {
    "id": 3492,
    "start_hour": 696,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D3492",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_30_255,LEG_30_198"
  },
  {
    "id": 3493,
    "start_hour": 715,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3493",
    "gerad_crew_id": "C0245",
    "flight_ids": "LEG_31_69,LEG_31_68,LEG_31_43"
  },
  {
    "id": 3494,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3494",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_29_106,LEG_29_99"
  },
  {
    "id": 3495,
    "start_hour": 688,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D3495",
    "gerad_crew_id": "C0246",
    "flight_ids": "LEG_30_121,LEG_30_219,LEG_30_90,LEG_30_185"
  },
  {
    "id": 3496,
    "start_hour": 681,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3496",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_29_265"
  },
  {
    "id": 3497,
    "start_hour": 706,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3497",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_30_46"
  },
  {
    "id": 3498,
    "start_hour": 711,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D3498",
    "gerad_crew_id": "C0247",
    "flight_ids": "LEG_31_201,LEG_31_89,LEG_31_36,LEG_31_50"
  },
  {
    "id": 3499,
    "start_hour": 674,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3499",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_29_242"
  },
  {
    "id": 3500,
    "start_hour": 704,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D3500",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_30_242"
  },
  {
    "id": 3501,
    "start_hour": 710,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D3501",
    "gerad_crew_id": "C0248",
    "flight_ids": "LEG_31_230,LEG_31_136,LEG_31_54"
  },
  {
    "id": 3502,
    "start_hour": 661,
    "duration_hours": 28,
    "required_skill": "A321",
    "gerad_duty_id": "D3502",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_29_49,LEG_29_20,LEG_29_4"
  },
  {
    "id": 3503,
    "start_hour": 701,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3503",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_30_77,LEG_30_76"
  },
  {
    "id": 3504,
    "start_hour": 709,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3504",
    "gerad_crew_id": "C0249",
    "flight_ids": "LEG_31_252"
  },
  {
    "id": 3505,
    "start_hour": 676,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3505",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_29_118,LEG_29_191,LEG_29_14"
  },
  {
    "id": 3506,
    "start_hour": 699,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3506",
    "gerad_crew_id": "C0250",
    "flight_ids": "LEG_30_161,LEG_30_244"
  },
  {
    "id": 3507,
    "start_hour": 682,
    "duration_hours": 2,
    "required_skill": "A319",
    "gerad_duty_id": "D3507",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_29_209"
  },
  {
    "id": 3508,
    "start_hour": 685,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3508",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_30_122,LEG_30_162,LEG_30_157"
  },
  {
    "id": 3509,
    "start_hour": 709,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D3509",
    "gerad_crew_id": "C0251",
    "flight_ids": "LEG_31_157,LEG_31_194,LEG_31_187,LEG_31_105"
  },
  {
    "id": 3510,
    "start_hour": 672,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D3510",
    "gerad_crew_id": "C0252",
    "flight_ids": "LEG_29_217,LEG_29_216,LEG_29_90,LEG_29_186"
  },
  {
    "id": 3511,
    "start_hour": 682,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3511",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_29_183"
  },
  {
    "id": 3512,
    "start_hour": 685,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3512",
    "gerad_crew_id": "C0253",
    "flight_ids": "LEG_30_166"
  },
  {
    "id": 3513,
    "start_hour": 672,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D3513",
    "gerad_crew_id": "C0254",
    "flight_ids": "LEG_29_208,LEG_29_266"
  },
  {
    "id": 3514,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D3514",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_29_41,LEG_29_48"
  },
  {
    "id": 3515,
    "start_hour": 660,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3515",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_29_185,LEG_29_184"
  },
  {
    "id": 3516,
    "start_hour": 672,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D3516",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_29_107,LEG_29_109"
  },
  {
    "id": 3517,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D3517",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_29_34,LEG_29_218"
  },
  {
    "id": 3518,
    "start_hour": 674,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3518",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_29_47"
  },
  {
    "id": 3519,
    "start_hour": 677,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D3519",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_29_211,LEG_29_125"
  },
  {
    "id": 3520,
    "start_hour": 685,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3520",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_30_17,LEG_30_21,LEG_30_57"
  },
  {
    "id": 3521,
    "start_hour": 720,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3521",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_31_24,LEG_31_134,LEG_31_225"
  },
  {
    "id": 3522,
    "start_hour": 678,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3522",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_29_38,LEG_29_42"
  },
  {
    "id": 3523,
    "start_hour": 660,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D3523",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_29_78,LEG_29_96,LEG_29_250"
  },
  {
    "id": 3524,
    "start_hour": 696,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D3524",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_30_82,LEG_30_31"
  },
  {
    "id": 3525,
    "start_hour": 715,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3525",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_31_108,LEG_31_217,LEG_31_216"
  },
  {
    "id": 3526,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3526",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_28_4"
  },
  {
    "id": 3527,
    "start_hour": 677,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D3527",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_29_77"
  },
  {
    "id": 3528,
    "start_hour": 685,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3528",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_30_221,LEG_30_123,LEG_30_95,LEG_30_130"
  },
  {
    "id": 3529,
    "start_hour": 709,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3529",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_31_261,LEG_31_7,LEG_31_135,LEG_31_138"
  },
  {
    "id": 3530,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3530",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_28_128,LEG_28_28"
  },
  {
    "id": 3531,
    "start_hour": 663,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3531",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_29_226"
  },
  {
    "id": 3532,
    "start_hour": 698,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3532",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_30_227,LEG_30_63"
  },
  {
    "id": 3533,
    "start_hour": 717,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3533",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_31_139,LEG_31_235,LEG_31_153"
  },
  {
    "id": 3534,
    "start_hour": 639,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3534",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_28_146"
  },
  {
    "id": 3535,
    "start_hour": 675,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3535",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_29_152,LEG_29_60"
  },
  {
    "id": 3536,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D3536",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_30_196,LEG_30_114"
  },
  {
    "id": 3537,
    "start_hour": 722,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3537",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_31_119,LEG_31_102,LEG_31_92"
  },
  {
    "id": 3538,
    "start_hour": 656,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D3538",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_28_160,LEG_28_237"
  },
  {
    "id": 3539,
    "start_hour": 663,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3539",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_29_239,LEG_29_62,LEG_29_149,LEG_29_204,LEG_29_114"
  },
  {
    "id": 3540,
    "start_hour": 698,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D3540",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_30_119,LEG_30_118,LEG_30_190,LEG_30_189"
  },
  {
    "id": 3541,
    "start_hour": 721,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3541",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_31_94"
  },
  {
    "id": 3542,
    "start_hour": 656,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D3542",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_28_169,LEG_28_250"
  },
  {
    "id": 3543,
    "start_hour": 664,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D3543",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_29_233,LEG_29_231,LEG_29_142"
  },
  {
    "id": 3544,
    "start_hour": 686,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3544",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_30_120"
  },
  {
    "id": 3545,
    "start_hour": 658,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D3545",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_28_181"
  },
  {
    "id": 3546,
    "start_hour": 661,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3546",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_29_122,LEG_29_163,LEG_29_158"
  },
  {
    "id": 3547,
    "start_hour": 685,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3547",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_30_100,LEG_30_187,LEG_30_258,LEG_30_126"
  },
  {
    "id": 3548,
    "start_hour": 709,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3548",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_31_71,LEG_31_103,LEG_31_215,LEG_31_145"
  },
  {
    "id": 3549,
    "start_hour": 637,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D3549",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_28_73,LEG_28_72"
  },
  {
    "id": 3550,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D3550",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_29_261,LEG_29_7,LEG_29_135,LEG_29_138"
  },
  {
    "id": 3551,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3551",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_28_84,LEG_28_226"
  },
  {
    "id": 3552,
    "start_hour": 663,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3552",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_29_98,LEG_29_268,LEG_29_166,LEG_29_164"
  },
  {
    "id": 3553,
    "start_hour": 637,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D3553",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_28_58,LEG_28_15,LEG_28_114"
  },
  {
    "id": 3554,
    "start_hour": 674,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3554",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_29_119,LEG_29_102,LEG_29_92"
  },
  {
    "id": 3555,
    "start_hour": 650,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D3555",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_28_247,LEG_28_2"
  },
  {
    "id": 3556,
    "start_hour": 652,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D3556",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_28_83,LEG_28_88"
  },
  {
    "id": 3557,
    "start_hour": 650,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D3557",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_28_139,LEG_28_140"
  },
  {
    "id": 3558,
    "start_hour": 641,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3558",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_28_111,LEG_28_167"
  },
  {
    "id": 3559,
    "start_hour": 649,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3559",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_28_260,LEG_28_261"
  },
  {
    "id": 3560,
    "start_hour": 640,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3560",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_28_154,LEG_28_132"
  },
  {
    "id": 3561,
    "start_hour": 649,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D3561",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_28_113,LEG_28_74"
  },
  {
    "id": 3562,
    "start_hour": 651,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3562",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_28_8,LEG_28_6"
  },
  {
    "id": 3563,
    "start_hour": 652,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D3563",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_28_25,LEG_28_18"
  },
  {
    "id": 3564,
    "start_hour": 649,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3564",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_28_264,LEG_28_180"
  },
  {
    "id": 3565,
    "start_hour": 653,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D3565",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_28_10,LEG_28_201"
  },
  {
    "id": 3566,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D3566",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_29_254,LEG_29_0"
  },
  {
    "id": 3567,
    "start_hour": 657,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3567",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_28_75"
  },
  {
    "id": 3568,
    "start_hour": 662,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D3568",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_29_70"
  },
  {
    "id": 3569,
    "start_hour": 656,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3569",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_28_150"
  },
  {
    "id": 3570,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D3570",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_29_150"
  },
  {
    "id": 3571,
    "start_hour": 657,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3571",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_28_178"
  },
  {
    "id": 3572,
    "start_hour": 661,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D3572",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_29_180"
  },
  {
    "id": 3573,
    "start_hour": 657,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D3573",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_28_3"
  },
  {
    "id": 3574,
    "start_hour": 661,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D3574",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_29_1"
  },
  {
    "id": 3575,
    "start_hour": 639,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D3575",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_28_145,LEG_28_81,LEG_28_141"
  },
  {
    "id": 3576,
    "start_hour": 662,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D3576",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_29_120"
  },
  {
    "id": 3577,
    "start_hour": 654,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D3577",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_28_153,LEG_28_155"
  },
  {
    "id": 3578,
    "start_hour": 654,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D3578",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_28_22,LEG_28_23"
  },
  {
    "id": 3579,
    "start_hour": 639,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3579",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_28_27,LEG_28_101"
  },
  {
    "id": 3580,
    "start_hour": 661,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D3580",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_29_127,LEG_29_165,LEG_29_95,LEG_29_130"
  },
  {
    "id": 3581,
    "start_hour": 684,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D3581",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_30_124,LEG_30_240,LEG_30_102,LEG_30_92"
  },
  {
    "id": 3582,
    "start_hour": 658,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D3582",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_28_80"
  },
  {
    "id": 3583,
    "start_hour": 660,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D3583",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_29_240,LEG_29_175,LEG_29_171,LEG_29_232,LEG_29_173"
  },
  {
    "id": 3584,
    "start_hour": 685,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D3584",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_30_127,LEG_30_164,LEG_30_110,LEG_30_233"
  }
];
