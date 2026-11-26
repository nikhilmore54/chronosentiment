// AUTO-GENERATED — do not edit by hand.
// Source: benchmarks/gerad-g2014-22/instance6/crew.csv + duties.csv
// Generator: scripts/gen_all_gerad_js.py
// Pipeline: GERAD instance6 → Roster/Duty/CrewMember → workers[]/shifts[]
//
// GERAD G-2014-22 Instance 6 (Kasirzadeh, Saddoune & Soumis 2014)
// 223 crew · 2476 duties · 761h horizon
// Normalization offset: 11h subtracted from all start_hour values.

export const GERAD_INSTANCE6_META = {
  "source": "GERAD G-2014-22 Instance 6 (Kasirzadeh, Saddoune & Soumis 2014)",
  "total_crew": 223,
  "total_duties": 2476,
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
export const GERAD_INSTANCE6_WORKERS = [
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
    "base": "BASE2",
    "gerad_id": "C0118",
    "contract_type": "full_time"
  },
  {
    "id": 119,
    "skills": [
      "A321"
    ],
    "name": "Anais Michel",
    "base": "BASE2",
    "gerad_id": "C0119",
    "contract_type": "full_time"
  },
  {
    "id": 120,
    "skills": [
      "A320"
    ],
    "name": "Thomas Martin",
    "base": "BASE2",
    "gerad_id": "C0120",
    "contract_type": "full_time"
  },
  {
    "id": 121,
    "skills": [
      "A320"
    ],
    "name": "Thomas Garcia",
    "base": "BASE2",
    "gerad_id": "C0121",
    "contract_type": "part_time"
  },
  {
    "id": 122,
    "skills": [
      "A319"
    ],
    "name": "Romain Robert",
    "base": "BASE2",
    "gerad_id": "C0122",
    "contract_type": "full_time"
  },
  {
    "id": 123,
    "skills": [
      "A319"
    ],
    "name": "Amandine Francois",
    "base": "BASE2",
    "gerad_id": "C0123",
    "contract_type": "full_time"
  },
  {
    "id": 124,
    "skills": [
      "A319"
    ],
    "name": "Clara Renard",
    "base": "BASE2",
    "gerad_id": "C0124",
    "contract_type": "part_time"
  },
  {
    "id": 125,
    "skills": [
      "A321"
    ],
    "name": "Adrien Leroy",
    "base": "BASE2",
    "gerad_id": "C0125",
    "contract_type": "part_time"
  },
  {
    "id": 126,
    "skills": [
      "A320"
    ],
    "name": "Anais Dumont",
    "base": "BASE2",
    "gerad_id": "C0126",
    "contract_type": "full_time"
  },
  {
    "id": 127,
    "skills": [
      "A321"
    ],
    "name": "Stephane David",
    "base": "BASE2",
    "gerad_id": "C0127",
    "contract_type": "full_time"
  },
  {
    "id": 128,
    "skills": [
      "A321"
    ],
    "name": "Michel Dubois",
    "base": "BASE2",
    "gerad_id": "C0128",
    "contract_type": "full_time"
  },
  {
    "id": 129,
    "skills": [
      "A319"
    ],
    "name": "Baptiste Collin",
    "base": "BASE2",
    "gerad_id": "C0129",
    "contract_type": "full_time"
  },
  {
    "id": 130,
    "skills": [
      "A320"
    ],
    "name": "Francois Caron",
    "base": "BASE2",
    "gerad_id": "C0130",
    "contract_type": "full_time"
  },
  {
    "id": 131,
    "skills": [
      "A319"
    ],
    "name": "Alexis Lefevre",
    "base": "BASE2",
    "gerad_id": "C0131",
    "contract_type": "full_time"
  },
  {
    "id": 132,
    "skills": [
      "A319"
    ],
    "name": "Emilie Garcia",
    "base": "BASE2",
    "gerad_id": "C0132",
    "contract_type": "full_time"
  },
  {
    "id": 133,
    "skills": [
      "A319"
    ],
    "name": "Clement Renard",
    "base": "BASE2",
    "gerad_id": "C0133",
    "contract_type": "full_time"
  },
  {
    "id": 134,
    "skills": [
      "A320"
    ],
    "name": "Laure Bernard",
    "base": "BASE2",
    "gerad_id": "C0134",
    "contract_type": "full_time"
  },
  {
    "id": 135,
    "skills": [
      "A321"
    ],
    "name": "Philippe Leclerc",
    "base": "BASE2",
    "gerad_id": "C0135",
    "contract_type": "full_time"
  },
  {
    "id": 136,
    "skills": [
      "A320"
    ],
    "name": "Raphael Bertrand",
    "base": "BASE2",
    "gerad_id": "C0136",
    "contract_type": "full_time"
  },
  {
    "id": 137,
    "skills": [
      "A319"
    ],
    "name": "Pierre Roux",
    "base": "BASE2",
    "gerad_id": "C0137",
    "contract_type": "full_time"
  },
  {
    "id": 138,
    "skills": [
      "A320"
    ],
    "name": "Manon Petit",
    "base": "BASE2",
    "gerad_id": "C0138",
    "contract_type": "full_time"
  },
  {
    "id": 139,
    "skills": [
      "A321"
    ],
    "name": "Manon Garnier",
    "base": "BASE2",
    "gerad_id": "C0139",
    "contract_type": "full_time"
  },
  {
    "id": 140,
    "skills": [
      "A319"
    ],
    "name": "Marie Muller",
    "base": "BASE2",
    "gerad_id": "C0140",
    "contract_type": "full_time"
  },
  {
    "id": 141,
    "skills": [
      "A319"
    ],
    "name": "In\u00e8s Martin",
    "base": "BASE2",
    "gerad_id": "C0141",
    "contract_type": "part_time"
  },
  {
    "id": 142,
    "skills": [
      "A319"
    ],
    "name": "Sylvie Chevalier",
    "base": "BASE2",
    "gerad_id": "C0142",
    "contract_type": "full_time"
  },
  {
    "id": 143,
    "skills": [
      "A321"
    ],
    "name": "Isabelle Fournier",
    "base": "BASE2",
    "gerad_id": "C0143",
    "contract_type": "full_time"
  },
  {
    "id": 144,
    "skills": [
      "A321"
    ],
    "name": "Manon Rousseau",
    "base": "BASE2",
    "gerad_id": "C0144",
    "contract_type": "part_time"
  },
  {
    "id": 145,
    "skills": [
      "A320"
    ],
    "name": "Emilie Bertrand",
    "base": "BASE2",
    "gerad_id": "C0145",
    "contract_type": "part_time"
  },
  {
    "id": 146,
    "skills": [
      "A319"
    ],
    "name": "Manon Roussel",
    "base": "BASE2",
    "gerad_id": "C0146",
    "contract_type": "full_time"
  },
  {
    "id": 147,
    "skills": [
      "A320"
    ],
    "name": "In\u00e8s Garnier",
    "base": "BASE2",
    "gerad_id": "C0147",
    "contract_type": "full_time"
  },
  {
    "id": 148,
    "skills": [
      "A321"
    ],
    "name": "Julie Michel",
    "base": "BASE2",
    "gerad_id": "C0148",
    "contract_type": "full_time"
  },
  {
    "id": 149,
    "skills": [
      "A320"
    ],
    "name": "Sandrine Martinez",
    "base": "BASE2",
    "gerad_id": "C0149",
    "contract_type": "full_time"
  },
  {
    "id": 150,
    "skills": [
      "A320"
    ],
    "name": "Adrien Renard",
    "base": "BASE2",
    "gerad_id": "C0150",
    "contract_type": "part_time"
  },
  {
    "id": 151,
    "skills": [
      "A320"
    ],
    "name": "Clara Morin",
    "base": "BASE2",
    "gerad_id": "C0151",
    "contract_type": "full_time"
  },
  {
    "id": 152,
    "skills": [
      "A321"
    ],
    "name": "Guillaume Roussel",
    "base": "BASE2",
    "gerad_id": "C0152",
    "contract_type": "full_time"
  },
  {
    "id": 153,
    "skills": [
      "A320"
    ],
    "name": "Emilie Mathieu",
    "base": "BASE2",
    "gerad_id": "C0153",
    "contract_type": "full_time"
  },
  {
    "id": 154,
    "skills": [
      "A319"
    ],
    "name": "Jade Bertrand",
    "base": "BASE2",
    "gerad_id": "C0154",
    "contract_type": "part_time"
  },
  {
    "id": 155,
    "skills": [
      "A321"
    ],
    "name": "Manon Morin",
    "base": "BASE2",
    "gerad_id": "C0155",
    "contract_type": "full_time"
  },
  {
    "id": 156,
    "skills": [
      "A321"
    ],
    "name": "Romain Bernard",
    "base": "BASE2",
    "gerad_id": "C0156",
    "contract_type": "full_time"
  },
  {
    "id": 157,
    "skills": [
      "A321"
    ],
    "name": "Marie Roussel",
    "base": "BASE2",
    "gerad_id": "C0157",
    "contract_type": "full_time"
  },
  {
    "id": 158,
    "skills": [
      "A319"
    ],
    "name": "Michel Durand",
    "base": "BASE2",
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
    "base": "BASE3",
    "gerad_id": "C0184",
    "contract_type": "part_time"
  },
  {
    "id": 185,
    "skills": [
      "A320"
    ],
    "name": "Clara Blanc",
    "base": "BASE3",
    "gerad_id": "C0185",
    "contract_type": "full_time"
  },
  {
    "id": 186,
    "skills": [
      "A321"
    ],
    "name": "Romain Thomas",
    "base": "BASE3",
    "gerad_id": "C0186",
    "contract_type": "full_time"
  },
  {
    "id": 187,
    "skills": [
      "A321"
    ],
    "name": "Quentin Petit",
    "base": "BASE3",
    "gerad_id": "C0187",
    "contract_type": "full_time"
  },
  {
    "id": 188,
    "skills": [
      "A321"
    ],
    "name": "Laurent Legrand",
    "base": "BASE3",
    "gerad_id": "C0188",
    "contract_type": "full_time"
  },
  {
    "id": 189,
    "skills": [
      "A319"
    ],
    "name": "Ines Guerin",
    "base": "BASE3",
    "gerad_id": "C0189",
    "contract_type": "full_time"
  },
  {
    "id": 190,
    "skills": [
      "A319"
    ],
    "name": "Julie Morin",
    "base": "BASE3",
    "gerad_id": "C0190",
    "contract_type": "full_time"
  },
  {
    "id": 191,
    "skills": [
      "A319"
    ],
    "name": "Stephane Masson",
    "base": "BASE3",
    "gerad_id": "C0191",
    "contract_type": "part_time"
  },
  {
    "id": 192,
    "skills": [
      "A320"
    ],
    "name": "Lucie Renard",
    "base": "BASE3",
    "gerad_id": "C0192",
    "contract_type": "full_time"
  },
  {
    "id": 193,
    "skills": [
      "A319"
    ],
    "name": "Clara Bertrand",
    "base": "BASE3",
    "gerad_id": "C0193",
    "contract_type": "full_time"
  },
  {
    "id": 194,
    "skills": [
      "A321"
    ],
    "name": "Mathieu Garnier",
    "base": "BASE3",
    "gerad_id": "C0194",
    "contract_type": "full_time"
  },
  {
    "id": 195,
    "skills": [
      "A320"
    ],
    "name": "Camille Masson",
    "base": "BASE3",
    "gerad_id": "C0195",
    "contract_type": "full_time"
  },
  {
    "id": 196,
    "skills": [
      "A320"
    ],
    "name": "Valerie Rousseau",
    "base": "BASE3",
    "gerad_id": "C0196",
    "contract_type": "full_time"
  },
  {
    "id": 197,
    "skills": [
      "A319"
    ],
    "name": "Florian Moreau",
    "base": "BASE3",
    "gerad_id": "C0197",
    "contract_type": "full_time"
  },
  {
    "id": 198,
    "skills": [
      "A319"
    ],
    "name": "Mathieu Durand",
    "base": "BASE3",
    "gerad_id": "C0198",
    "contract_type": "full_time"
  },
  {
    "id": 199,
    "skills": [
      "A321"
    ],
    "name": "Adrien Giraud",
    "base": "BASE3",
    "gerad_id": "C0199",
    "contract_type": "full_time"
  },
  {
    "id": 200,
    "skills": [
      "A321"
    ],
    "name": "Guillaume Petit",
    "base": "BASE3",
    "gerad_id": "C0200",
    "contract_type": "full_time"
  },
  {
    "id": 201,
    "skills": [
      "A321"
    ],
    "name": "Thibault Giraud",
    "base": "BASE3",
    "gerad_id": "C0201",
    "contract_type": "part_time"
  },
  {
    "id": 202,
    "skills": [
      "A320"
    ],
    "name": "Oceane Leroy",
    "base": "BASE3",
    "gerad_id": "C0202",
    "contract_type": "full_time"
  },
  {
    "id": 203,
    "skills": [
      "A319"
    ],
    "name": "Florian Bonnet",
    "base": "BASE3",
    "gerad_id": "C0203",
    "contract_type": "full_time"
  },
  {
    "id": 204,
    "skills": [
      "A320"
    ],
    "name": "Manon Masson",
    "base": "BASE3",
    "gerad_id": "C0204",
    "contract_type": "full_time"
  },
  {
    "id": 205,
    "skills": [
      "A320"
    ],
    "name": "Alexis Morin",
    "base": "BASE3",
    "gerad_id": "C0205",
    "contract_type": "part_time"
  },
  {
    "id": 206,
    "skills": [
      "A319"
    ],
    "name": "Laure Roux",
    "base": "BASE3",
    "gerad_id": "C0206",
    "contract_type": "full_time"
  },
  {
    "id": 207,
    "skills": [
      "A319"
    ],
    "name": "Sylvie Garcia",
    "base": "BASE3",
    "gerad_id": "C0207",
    "contract_type": "full_time"
  },
  {
    "id": 208,
    "skills": [
      "A321"
    ],
    "name": "Mathieu Mercier",
    "base": "BASE3",
    "gerad_id": "C0208",
    "contract_type": "full_time"
  },
  {
    "id": 209,
    "skills": [
      "A320"
    ],
    "name": "Sebastien Lopez",
    "base": "BASE3",
    "gerad_id": "C0209",
    "contract_type": "full_time"
  },
  {
    "id": 210,
    "skills": [
      "A321"
    ],
    "name": "Pierre Collin",
    "base": "BASE3",
    "gerad_id": "C0210",
    "contract_type": "full_time"
  },
  {
    "id": 211,
    "skills": [
      "A319"
    ],
    "name": "Damien Clement",
    "base": "BASE3",
    "gerad_id": "C0211",
    "contract_type": "full_time"
  },
  {
    "id": 212,
    "skills": [
      "A319"
    ],
    "name": "Anais Robert",
    "base": "BASE3",
    "gerad_id": "C0212",
    "contract_type": "full_time"
  },
  {
    "id": 213,
    "skills": [
      "A321"
    ],
    "name": "Nicolas Laurent",
    "base": "BASE3",
    "gerad_id": "C0213",
    "contract_type": "full_time"
  },
  {
    "id": 214,
    "skills": [
      "A320"
    ],
    "name": "Alexis Dupont",
    "base": "BASE3",
    "gerad_id": "C0214",
    "contract_type": "full_time"
  },
  {
    "id": 215,
    "skills": [
      "A319"
    ],
    "name": "Aurelie Garcia",
    "base": "BASE3",
    "gerad_id": "C0215",
    "contract_type": "part_time"
  },
  {
    "id": 216,
    "skills": [
      "A320"
    ],
    "name": "Lea Dubois",
    "base": "BASE3",
    "gerad_id": "C0216",
    "contract_type": "full_time"
  },
  {
    "id": 217,
    "skills": [
      "A319"
    ],
    "name": "Celine Francois",
    "base": "BASE3",
    "gerad_id": "C0217",
    "contract_type": "full_time"
  },
  {
    "id": 218,
    "skills": [
      "A319"
    ],
    "name": "Sophie Lopez",
    "base": "BASE3",
    "gerad_id": "C0218",
    "contract_type": "full_time"
  },
  {
    "id": 219,
    "skills": [
      "A319"
    ],
    "name": "Catherine Roussel",
    "base": "BASE3",
    "gerad_id": "C0219",
    "contract_type": "full_time"
  },
  {
    "id": 220,
    "skills": [
      "A320"
    ],
    "name": "Clara Schmitt",
    "base": "BASE3",
    "gerad_id": "C0220",
    "contract_type": "part_time"
  },
  {
    "id": 221,
    "skills": [
      "A319"
    ],
    "name": "Oceane Mercier",
    "base": "BASE3",
    "gerad_id": "C0221",
    "contract_type": "full_time"
  },
  {
    "id": 222,
    "skills": [
      "A320"
    ],
    "name": "Elise Nicolas",
    "base": "BASE3",
    "gerad_id": "C0222",
    "contract_type": "part_time"
  },
  {
    "id": 223,
    "skills": [
      "A319"
    ],
    "name": "Aurelie Petit",
    "base": "BASE3",
    "gerad_id": "C0223",
    "contract_type": "part_time"
  }
];

// shifts[]: each GERAD Duty projected to UltraCrew Shift schema.
// id: numeric duty_id, start_hour: normalized FDP report time,
// duration_hours: FDP length (release - report), required_skill: crew qualification.
export const GERAD_INSTANCE6_SHIFTS = [
  {
    "id": 1,
    "start_hour": 660,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0001",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_29_5,LEG_29_8"
  },
  {
    "id": 2,
    "start_hour": 684,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0002",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_30_49,LEG_30_202,LEG_30_186"
  },
  {
    "id": 3,
    "start_hour": 678,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0003",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_29_59,LEG_29_20"
  },
  {
    "id": 4,
    "start_hour": 686,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0004",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_30_4,LEG_30_13,LEG_30_205"
  },
  {
    "id": 5,
    "start_hour": 724,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0005",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_31_202,LEG_31_203,LEG_31_187"
  },
  {
    "id": 6,
    "start_hour": 676,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0006",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_29_159,LEG_29_126"
  },
  {
    "id": 7,
    "start_hour": 685,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0007",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_30_79,LEG_30_114,LEG_30_123,LEG_30_196"
  },
  {
    "id": 8,
    "start_hour": 722,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0008",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_31_105,LEG_31_180"
  },
  {
    "id": 9,
    "start_hour": 674,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0009",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_29_191,LEG_29_189,LEG_29_97"
  },
  {
    "id": 10,
    "start_hour": 684,
    "duration_hours": 2,
    "required_skill": "A320",
    "gerad_duty_id": "D0010",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_30_167"
  },
  {
    "id": 11,
    "start_hour": 681,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0011",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_29_106"
  },
  {
    "id": 12,
    "start_hour": 684,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0012",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_30_109"
  },
  {
    "id": 13,
    "start_hour": 681,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0013",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_29_18"
  },
  {
    "id": 14,
    "start_hour": 714,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0014",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_31_16"
  },
  {
    "id": 15,
    "start_hour": 674,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0015",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_29_101,LEG_29_53"
  },
  {
    "id": 16,
    "start_hour": 675,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0016",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_29_177"
  },
  {
    "id": 17,
    "start_hour": 704,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0017",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_30_178"
  },
  {
    "id": 18,
    "start_hour": 677,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0018",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_29_194,LEG_29_187"
  },
  {
    "id": 19,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0019",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_29_178,LEG_29_132"
  },
  {
    "id": 20,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0020",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_29_108,LEG_29_102"
  },
  {
    "id": 21,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0021",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_29_186,LEG_29_176"
  },
  {
    "id": 22,
    "start_hour": 672,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0022",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_29_58,LEG_29_61"
  },
  {
    "id": 23,
    "start_hour": 662,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0023",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_29_76,LEG_29_190"
  },
  {
    "id": 24,
    "start_hour": 674,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0024",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_29_131,LEG_29_98"
  },
  {
    "id": 25,
    "start_hour": 679,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0025",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_29_192,LEG_29_185"
  },
  {
    "id": 26,
    "start_hour": 672,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0026",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_29_15,LEG_29_30,LEG_29_84"
  },
  {
    "id": 27,
    "start_hour": 697,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0027",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_30_41,LEG_30_203,LEG_30_106"
  },
  {
    "id": 28,
    "start_hour": 723,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0028",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_31_150,LEG_31_12"
  },
  {
    "id": 29,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0029",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_05_59,LEG_05_117"
  },
  {
    "id": 30,
    "start_hour": 110,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0030",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_06_122,LEG_06_143,LEG_06_172"
  },
  {
    "id": 31,
    "start_hour": 154,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0031",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_07_191"
  },
  {
    "id": 32,
    "start_hour": 159,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0032",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_08_3"
  },
  {
    "id": 33,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0033",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_05_97,LEG_05_160,LEG_05_164"
  },
  {
    "id": 34,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0034",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_06_164,LEG_06_42"
  },
  {
    "id": 35,
    "start_hour": 144,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0035",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_07_7,LEG_07_170"
  },
  {
    "id": 36,
    "start_hour": 168,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0036",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_08_175,LEG_08_176,LEG_08_38"
  },
  {
    "id": 37,
    "start_hour": 98,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0037",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_05_66,LEG_05_178,LEG_05_37"
  },
  {
    "id": 38,
    "start_hour": 108,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0038",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_06_155"
  },
  {
    "id": 39,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0039",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_05_34,LEG_05_32"
  },
  {
    "id": 40,
    "start_hour": 86,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0040",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_05_31,LEG_05_100"
  },
  {
    "id": 41,
    "start_hour": 125,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0041",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_06_44,LEG_06_48"
  },
  {
    "id": 42,
    "start_hour": 109,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0042",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_06_124,LEG_06_37"
  },
  {
    "id": 43,
    "start_hour": 109,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0043",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_06_47,LEG_06_40"
  },
  {
    "id": 44,
    "start_hour": 128,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0044",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_06_158"
  },
  {
    "id": 45,
    "start_hour": 121,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0045",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_06_147,LEG_06_145"
  },
  {
    "id": 46,
    "start_hour": 123,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0046",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_06_94,LEG_06_115,LEG_06_5"
  },
  {
    "id": 47,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0047",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_07_138,LEG_07_182,LEG_07_185"
  },
  {
    "id": 48,
    "start_hour": 131,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0048",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_06_11"
  },
  {
    "id": 49,
    "start_hour": 148,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0049",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_07_12"
  },
  {
    "id": 50,
    "start_hour": 128,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0050",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_06_97,LEG_06_98"
  },
  {
    "id": 51,
    "start_hour": 121,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0051",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_06_162,LEG_06_160,LEG_06_146,LEG_06_71"
  },
  {
    "id": 52,
    "start_hour": 126,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0052",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_06_39,LEG_06_45,LEG_06_46"
  },
  {
    "id": 53,
    "start_hour": 146,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0053",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_07_41,LEG_07_187"
  },
  {
    "id": 54,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0054",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_08_181,LEG_08_42,LEG_08_50"
  },
  {
    "id": 55,
    "start_hour": 123,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0055",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_06_144,LEG_06_66,LEG_06_19"
  },
  {
    "id": 56,
    "start_hour": 145,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0056",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_07_143,LEG_07_188"
  },
  {
    "id": 57,
    "start_hour": 168,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0057",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_08_191,LEG_08_171"
  },
  {
    "id": 58,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0058",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_09_181,LEG_09_180,LEG_09_9"
  },
  {
    "id": 59,
    "start_hour": 109,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0059",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_06_157,LEG_06_29"
  },
  {
    "id": 60,
    "start_hour": 134,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0060",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_07_146,LEG_07_151,LEG_07_4"
  },
  {
    "id": 61,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0061",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_08_139,LEG_08_79,LEG_08_108,LEG_08_161,LEG_08_80"
  },
  {
    "id": 62,
    "start_hour": 129,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0062",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_06_123,LEG_06_125"
  },
  {
    "id": 63,
    "start_hour": 133,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0063",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_07_81"
  },
  {
    "id": 64,
    "start_hour": 168,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0064",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_08_78,LEG_08_140,LEG_08_55,LEG_08_185"
  },
  {
    "id": 65,
    "start_hour": 192,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0065",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_09_175,LEG_09_140,LEG_09_55,LEG_09_161,LEG_09_80"
  },
  {
    "id": 66,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0066",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_11_61,LEG_11_107,LEG_11_178"
  },
  {
    "id": 67,
    "start_hour": 265,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0067",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_12_44,LEG_12_13"
  },
  {
    "id": 68,
    "start_hour": 290,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0068",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_13_7,LEG_13_120"
  },
  {
    "id": 69,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0069",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_14_118,LEG_14_26,LEG_14_59"
  },
  {
    "id": 70,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0070",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_11_63,LEG_11_118"
  },
  {
    "id": 71,
    "start_hour": 254,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0071",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_12_128,LEG_12_148,LEG_12_177"
  },
  {
    "id": 72,
    "start_hour": 298,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0072",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_13_177"
  },
  {
    "id": 73,
    "start_hour": 303,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0073",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_14_3"
  },
  {
    "id": 74,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0074",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_11_103,LEG_11_166,LEG_11_16"
  },
  {
    "id": 75,
    "start_hour": 263,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0075",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_12_20,LEG_12_159"
  },
  {
    "id": 76,
    "start_hour": 291,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0076",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_13_153,LEG_13_133"
  },
  {
    "id": 77,
    "start_hour": 312,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0077",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_14_121,LEG_14_129,LEG_14_73"
  },
  {
    "id": 78,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0078",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_11_65,LEG_11_66"
  },
  {
    "id": 79,
    "start_hour": 239,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0079",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_11_23,LEG_11_130,LEG_11_72"
  },
  {
    "id": 80,
    "start_hour": 230,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0080",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_11_34,LEG_11_104"
  },
  {
    "id": 81,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0081",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_01_81,LEG_01_84"
  },
  {
    "id": 82,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0082",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_01_134,LEG_01_135"
  },
  {
    "id": 83,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0083",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_01_1,LEG_01_4"
  },
  {
    "id": 84,
    "start_hour": 9,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0084",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_01_79,LEG_01_58"
  },
  {
    "id": 85,
    "start_hour": 14,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0085",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_02_24"
  },
  {
    "id": 86,
    "start_hour": 7,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0086",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_01_15,LEG_01_14"
  },
  {
    "id": 87,
    "start_hour": 14,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0087",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_02_124,LEG_02_123"
  },
  {
    "id": 88,
    "start_hour": 3,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0088",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_01_148,LEG_01_74,LEG_01_13"
  },
  {
    "id": 89,
    "start_hour": 23,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0089",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_02_19,LEG_02_101,LEG_02_99"
  },
  {
    "id": 90,
    "start_hour": 2,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0090",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_01_86,LEG_01_85,LEG_01_167"
  },
  {
    "id": 91,
    "start_hour": 24,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0091",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_02_190"
  },
  {
    "id": 92,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0092",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_01_76,LEG_01_107"
  },
  {
    "id": 93,
    "start_hour": 14,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0093",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_02_134,LEG_02_156"
  },
  {
    "id": 94,
    "start_hour": 3,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0094",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_01_26,LEG_01_25,LEG_01_96"
  },
  {
    "id": 95,
    "start_hour": 24,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0095",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_02_100,LEG_02_154,LEG_02_155"
  },
  {
    "id": 96,
    "start_hour": 6,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0096",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_01_100,LEG_01_67"
  },
  {
    "id": 97,
    "start_hour": 13,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D0097",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_02_8,LEG_02_104,LEG_02_158,LEG_02_75"
  },
  {
    "id": 98,
    "start_hour": 3,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0098",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_01_122,LEG_01_121"
  },
  {
    "id": 99,
    "start_hour": 12,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0099",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_02_84,LEG_02_93,LEG_02_167,LEG_02_85"
  },
  {
    "id": 100,
    "start_hour": 5,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0100",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_01_104,LEG_01_106,LEG_01_22"
  },
  {
    "id": 101,
    "start_hour": 24,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0101",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_02_166,LEG_02_20,LEG_02_15"
  },
  {
    "id": 102,
    "start_hour": 4,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0102",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_01_113,LEG_01_21"
  },
  {
    "id": 103,
    "start_hour": 12,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0103",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_02_13,LEG_02_0,LEG_02_131,LEG_02_25"
  },
  {
    "id": 104,
    "start_hour": 0,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0104",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_01_145,LEG_01_55"
  },
  {
    "id": 105,
    "start_hour": 24,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0105",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_02_60,LEG_02_102,LEG_02_165"
  },
  {
    "id": 106,
    "start_hour": 7,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0106",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_01_66,LEG_01_115"
  },
  {
    "id": 107,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0107",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_01_103,LEG_01_78"
  },
  {
    "id": 108,
    "start_hour": 5,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0108",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_01_5,LEG_01_2"
  },
  {
    "id": 109,
    "start_hour": 10,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0109",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_01_169"
  },
  {
    "id": 110,
    "start_hour": 14,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0110",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_02_146,LEG_02_151"
  },
  {
    "id": 111,
    "start_hour": 36,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0111",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_03_13,LEG_03_0,LEG_03_132,LEG_03_25"
  },
  {
    "id": 112,
    "start_hour": 10,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0112",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_01_83"
  },
  {
    "id": 113,
    "start_hour": 26,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0113",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_02_96,LEG_02_26"
  },
  {
    "id": 114,
    "start_hour": 48,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0114",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_03_167,LEG_03_20,LEG_03_15"
  },
  {
    "id": 115,
    "start_hour": 8,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0115",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_01_130,LEG_01_126"
  },
  {
    "id": 116,
    "start_hour": 14,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0116",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_02_89,LEG_02_92"
  },
  {
    "id": 117,
    "start_hour": 37,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D0117",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_03_8,LEG_03_105,LEG_03_159,LEG_03_75"
  },
  {
    "id": 118,
    "start_hour": 5,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0118",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_01_125,LEG_01_133,LEG_01_24"
  },
  {
    "id": 119,
    "start_hour": 24,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0119",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_02_81,LEG_02_130"
  },
  {
    "id": 120,
    "start_hour": 49,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0120",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_03_133,LEG_03_83,LEG_03_84"
  },
  {
    "id": 121,
    "start_hour": 0,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0121",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_01_11,LEG_01_31"
  },
  {
    "id": 122,
    "start_hour": 15,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0122",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_02_68,LEG_02_157"
  },
  {
    "id": 123,
    "start_hour": 37,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0123",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_03_173,LEG_03_31"
  },
  {
    "id": 124,
    "start_hour": 10,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0124",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_01_108"
  },
  {
    "id": 125,
    "start_hour": 24,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0125",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_02_128,LEG_02_188"
  },
  {
    "id": 126,
    "start_hour": 48,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0126",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_03_191"
  },
  {
    "id": 127,
    "start_hour": 8,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0127",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_01_23,LEG_01_50"
  },
  {
    "id": 128,
    "start_hour": 14,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0128",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_02_67,LEG_02_64,LEG_02_63,LEG_02_117"
  },
  {
    "id": 129,
    "start_hour": 6,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0129",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_01_82,LEG_01_149,LEG_01_18"
  },
  {
    "id": 130,
    "start_hour": 25,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0130",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_02_143,LEG_02_87,LEG_02_125"
  },
  {
    "id": 131,
    "start_hour": 5,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0131",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_01_98,LEG_01_95,LEG_01_168"
  },
  {
    "id": 132,
    "start_hour": 34,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0132",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_02_191"
  },
  {
    "id": 133,
    "start_hour": 7,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0133",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_01_151,LEG_01_101"
  },
  {
    "id": 134,
    "start_hour": 14,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0134",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_02_116,LEG_02_114,LEG_02_76,LEG_02_133"
  },
  {
    "id": 135,
    "start_hour": 3,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0135",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_01_17,LEG_01_12,LEG_01_75"
  },
  {
    "id": 136,
    "start_hour": 26,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0136",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_02_88,LEG_02_169"
  },
  {
    "id": 137,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0137",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_03_113,LEG_03_96,LEG_03_169,LEG_03_16"
  },
  {
    "id": 138,
    "start_hour": 71,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0138",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_04_19,LEG_04_102,LEG_04_100"
  },
  {
    "id": 139,
    "start_hour": 8,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0139",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_01_136,LEG_01_132"
  },
  {
    "id": 140,
    "start_hour": 25,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0140",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_02_32,LEG_02_149"
  },
  {
    "id": 141,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0141",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_03_148,LEG_03_189"
  },
  {
    "id": 142,
    "start_hour": 72,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0142",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_04_191"
  },
  {
    "id": 143,
    "start_hour": 5,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0143",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_01_72,LEG_01_73,LEG_01_120"
  },
  {
    "id": 144,
    "start_hour": 24,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0144",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_02_121,LEG_02_120"
  },
  {
    "id": 145,
    "start_hour": 48,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0145",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_03_129,LEG_03_131"
  },
  {
    "id": 146,
    "start_hour": 73,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0146",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_04_133,LEG_04_83,LEG_04_84"
  },
  {
    "id": 147,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0147",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_07_63,LEG_07_117"
  },
  {
    "id": 148,
    "start_hour": 158,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0148",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_08_147,LEG_08_152,LEG_08_87"
  },
  {
    "id": 149,
    "start_hour": 194,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0149",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_09_89,LEG_09_98"
  },
  {
    "id": 150,
    "start_hour": 218,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0150",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_10_97,LEG_10_27,LEG_10_58"
  },
  {
    "id": 151,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0151",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_07_65,LEG_07_66"
  },
  {
    "id": 152,
    "start_hour": 158,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0152",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_08_67,LEG_08_164,LEG_08_141"
  },
  {
    "id": 153,
    "start_hour": 192,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0153",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_09_122,LEG_09_171"
  },
  {
    "id": 154,
    "start_hour": 216,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0154",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_10_175,LEG_10_176,LEG_10_38"
  },
  {
    "id": 155,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0155",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_07_148,LEG_07_149"
  },
  {
    "id": 156,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0156",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_08_148,LEG_08_27,LEG_08_58"
  },
  {
    "id": 157,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0157",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_07_61,LEG_07_106,LEG_07_11"
  },
  {
    "id": 158,
    "start_hour": 167,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0158",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_08_162,LEG_08_39,LEG_08_62"
  },
  {
    "id": 159,
    "start_hour": 134,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0159",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_07_34,LEG_07_103"
  },
  {
    "id": 160,
    "start_hour": 146,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0160",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_07_33,LEG_07_38"
  },
  {
    "id": 161,
    "start_hour": 720,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0161",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_31_11,LEG_31_9,LEG_31_183"
  },
  {
    "id": 162,
    "start_hour": 720,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0162",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_31_58,LEG_31_68,LEG_31_51"
  },
  {
    "id": 163,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0163",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_31_186,LEG_31_176"
  },
  {
    "id": 164,
    "start_hour": 722,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0164",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_31_101,LEG_31_56"
  },
  {
    "id": 165,
    "start_hour": 722,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0165",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_31_191,LEG_31_189"
  },
  {
    "id": 166,
    "start_hour": 708,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0166",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_31_5,LEG_31_8"
  },
  {
    "id": 167,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0167",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_31_178,LEG_31_132"
  },
  {
    "id": 168,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0168",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_31_76,LEG_31_190"
  },
  {
    "id": 169,
    "start_hour": 710,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0169",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_31_108,LEG_31_102"
  },
  {
    "id": 170,
    "start_hour": 722,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0170",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_31_131,LEG_31_98"
  },
  {
    "id": 171,
    "start_hour": 723,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0171",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_31_177"
  },
  {
    "id": 172,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0172",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_09_95,LEG_09_99"
  },
  {
    "id": 173,
    "start_hour": 183,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0173",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_09_3,LEG_09_23"
  },
  {
    "id": 174,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0174",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_09_1,LEG_09_5"
  },
  {
    "id": 175,
    "start_hour": 195,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0175",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_09_143,LEG_09_142"
  },
  {
    "id": 176,
    "start_hour": 204,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0176",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_10_85,LEG_10_94,LEG_10_168,LEG_10_86"
  },
  {
    "id": 177,
    "start_hour": 195,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0177",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_09_30,LEG_09_29,LEG_09_26"
  },
  {
    "id": 178,
    "start_hour": 216,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0178",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_10_167,LEG_10_20,LEG_10_15"
  },
  {
    "id": 179,
    "start_hour": 201,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0179",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_09_92,LEG_09_69"
  },
  {
    "id": 180,
    "start_hour": 206,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0180",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_10_24"
  },
  {
    "id": 181,
    "start_hour": 192,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0181",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_09_14,LEG_09_36,LEG_09_73"
  },
  {
    "id": 182,
    "start_hour": 215,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0182",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_10_71,LEG_10_33,LEG_10_149"
  },
  {
    "id": 183,
    "start_hour": 196,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0183",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_09_132,LEG_09_25"
  },
  {
    "id": 184,
    "start_hour": 205,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D0184",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_10_8,LEG_10_105,LEG_10_159,LEG_10_75"
  },
  {
    "id": 185,
    "start_hour": 197,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0185",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_09_6,LEG_09_2"
  },
  {
    "id": 186,
    "start_hour": 182,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0186",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_09_117,LEG_09_115,LEG_09_76,LEG_09_134"
  },
  {
    "id": 187,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0187",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_09_120,LEG_09_91"
  },
  {
    "id": 188,
    "start_hour": 182,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0188",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_09_147,LEG_09_152"
  },
  {
    "id": 189,
    "start_hour": 204,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0189",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_10_123,LEG_10_113,LEG_10_189"
  },
  {
    "id": 190,
    "start_hour": 240,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0190",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_11_191"
  },
  {
    "id": 191,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0191",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_09_88,LEG_09_126"
  },
  {
    "id": 192,
    "start_hour": 206,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0192",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_10_135,LEG_10_157,LEG_10_21"
  },
  {
    "id": 193,
    "start_hour": 241,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0193",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_11_144,LEG_11_114,LEG_11_111"
  },
  {
    "id": 194,
    "start_hour": 192,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0194",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_09_165,LEG_09_64"
  },
  {
    "id": 195,
    "start_hour": 207,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0195",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_10_68,LEG_10_158"
  },
  {
    "id": 196,
    "start_hour": 229,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0196",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_11_173,LEG_11_31"
  },
  {
    "id": 197,
    "start_hour": 180,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D0197",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_09_123,LEG_09_166,LEG_09_141"
  },
  {
    "id": 198,
    "start_hour": 216,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0198",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_10_122,LEG_10_131"
  },
  {
    "id": 199,
    "start_hour": 241,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0199",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_11_133,LEG_11_83,LEG_11_84"
  },
  {
    "id": 200,
    "start_hour": 182,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0200",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_09_90,LEG_09_93,LEG_09_21"
  },
  {
    "id": 201,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0201",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_10_41,LEG_10_46"
  },
  {
    "id": 202,
    "start_hour": 240,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0202",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_11_7"
  },
  {
    "id": 203,
    "start_hour": 182,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0203",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_09_125,LEG_09_124,LEG_09_153"
  },
  {
    "id": 204,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0204",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_10_32,LEG_10_96,LEG_10_169,LEG_10_112"
  },
  {
    "id": 205,
    "start_hour": 240,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0205",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_11_101,LEG_11_155,LEG_11_156"
  },
  {
    "id": 206,
    "start_hour": 199,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0206",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_09_18,LEG_09_17"
  },
  {
    "id": 207,
    "start_hour": 206,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0207",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_10_125,LEG_10_124,LEG_10_153"
  },
  {
    "id": 208,
    "start_hour": 241,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0208",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_11_32,LEG_11_96,LEG_11_169,LEG_11_150"
  },
  {
    "id": 209,
    "start_hour": 266,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0209",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_12_138"
  },
  {
    "id": 210,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0210",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_09_151,LEG_09_146"
  },
  {
    "id": 211,
    "start_hour": 206,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0211",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_10_90,LEG_10_93,LEG_10_141"
  },
  {
    "id": 212,
    "start_hour": 240,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0212",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_11_122,LEG_11_131"
  },
  {
    "id": 213,
    "start_hour": 265,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0213",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_12_126,LEG_12_76,LEG_12_77"
  },
  {
    "id": 214,
    "start_hour": 197,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0214",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_09_145,LEG_09_154,LEG_09_28"
  },
  {
    "id": 215,
    "start_hour": 216,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0215",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_10_82,LEG_10_121"
  },
  {
    "id": 216,
    "start_hour": 240,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0216",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_11_129,LEG_11_121"
  },
  {
    "id": 217,
    "start_hour": 264,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0217",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_12_122,LEG_12_92,LEG_12_94"
  },
  {
    "id": 218,
    "start_hour": 563,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0218",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_24_10"
  },
  {
    "id": 219,
    "start_hour": 580,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0219",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_25_12,LEG_25_140,LEG_25_142"
  },
  {
    "id": 220,
    "start_hour": 624,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0220",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_27_132,LEG_27_106,LEG_27_107"
  },
  {
    "id": 221,
    "start_hour": 555,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0221",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_24_108,LEG_24_132,LEG_24_22"
  },
  {
    "id": 222,
    "start_hour": 577,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0222",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_25_148,LEG_25_175"
  },
  {
    "id": 223,
    "start_hour": 600,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0223",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_26_166,LEG_26_13"
  },
  {
    "id": 224,
    "start_hour": 629,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0224",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_27_9"
  },
  {
    "id": 225,
    "start_hour": 561,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0225",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_24_140,LEG_24_142"
  },
  {
    "id": 226,
    "start_hour": 565,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0226",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_25_83"
  },
  {
    "id": 227,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0227",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_26_77,LEG_26_43,LEG_26_50"
  },
  {
    "id": 228,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0228",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_24_45,LEG_24_16,LEG_24_174"
  },
  {
    "id": 229,
    "start_hour": 577,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0229",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_25_117,LEG_25_125"
  },
  {
    "id": 230,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0230",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_26_127,LEG_26_130,LEG_26_104"
  },
  {
    "id": 231,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0231",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_24_41,LEG_24_64,LEG_24_75"
  },
  {
    "id": 232,
    "start_hour": 575,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0232",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_25_73,LEG_25_72,LEG_25_188"
  },
  {
    "id": 233,
    "start_hour": 541,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0233",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_24_141,LEG_24_42"
  },
  {
    "id": 234,
    "start_hour": 560,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0234",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_24_56,LEG_24_51"
  },
  {
    "id": 235,
    "start_hour": 553,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0235",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_24_167,LEG_24_164"
  },
  {
    "id": 236,
    "start_hour": 560,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0236",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_24_113,LEG_24_114"
  },
  {
    "id": 237,
    "start_hour": 556,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0237",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_24_187,LEG_24_190"
  },
  {
    "id": 238,
    "start_hour": 541,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0238",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_24_54,LEG_24_46"
  },
  {
    "id": 239,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0239",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_24_183,LEG_24_181,LEG_24_165,LEG_24_82"
  },
  {
    "id": 240,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0240",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_08_63,LEG_08_118"
  },
  {
    "id": 241,
    "start_hour": 182,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0241",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_09_135,LEG_09_157,LEG_09_190"
  },
  {
    "id": 242,
    "start_hour": 226,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0242",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_10_192"
  },
  {
    "id": 243,
    "start_hour": 231,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0243",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_11_3"
  },
  {
    "id": 244,
    "start_hour": 174,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0244",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_08_74,LEG_08_22,LEG_08_188"
  },
  {
    "id": 245,
    "start_hour": 195,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0245",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_09_164,LEG_09_27,LEG_09_58"
  },
  {
    "id": 246,
    "start_hour": 170,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0246",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_08_33,LEG_08_149"
  },
  {
    "id": 247,
    "start_hour": 180,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0247",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_09_13,LEG_09_0,LEG_09_130,LEG_09_72"
  },
  {
    "id": 248,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0248",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_08_61,LEG_08_107,LEG_08_11"
  },
  {
    "id": 249,
    "start_hour": 191,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0249",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_09_162,LEG_09_39,LEG_09_62"
  },
  {
    "id": 250,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0250",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_08_65,LEG_08_66"
  },
  {
    "id": 251,
    "start_hour": 158,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0251",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_08_34,LEG_08_104"
  },
  {
    "id": 252,
    "start_hour": 167,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0252",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_08_23,LEG_08_130,LEG_08_72"
  },
  {
    "id": 253,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0253",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_04_103,LEG_04_57,LEG_04_56"
  },
  {
    "id": 254,
    "start_hour": 99,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0254",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_05_56,LEG_05_70,LEG_05_21,LEG_05_181"
  },
  {
    "id": 255,
    "start_hour": 123,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0255",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_06_148,LEG_06_26"
  },
  {
    "id": 256,
    "start_hour": 144,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0256",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_07_82,LEG_07_129,LEG_07_72"
  },
  {
    "id": 257,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0257",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_04_63,LEG_04_118"
  },
  {
    "id": 258,
    "start_hour": 86,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0258",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_05_143,LEG_05_148,LEG_05_82"
  },
  {
    "id": 259,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0259",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_06_78,LEG_06_25,LEG_06_52"
  },
  {
    "id": 260,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0260",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_04_65,LEG_04_66"
  },
  {
    "id": 261,
    "start_hour": 86,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0261",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_05_63,LEG_05_60,LEG_05_61,LEG_05_62"
  },
  {
    "id": 262,
    "start_hour": 74,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0262",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_04_33,LEG_04_149"
  },
  {
    "id": 263,
    "start_hour": 84,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0263",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_05_14,LEG_05_0,LEG_05_129,LEG_05_68"
  },
  {
    "id": 264,
    "start_hour": 62,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0264",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_04_34,LEG_04_104"
  },
  {
    "id": 265,
    "start_hour": 133,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0265",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_07_136,LEG_07_40"
  },
  {
    "id": 266,
    "start_hour": 133,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0266",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_07_52,LEG_07_44"
  },
  {
    "id": 267,
    "start_hour": 145,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0267",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_07_162,LEG_07_159"
  },
  {
    "id": 268,
    "start_hour": 145,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0268",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_07_43,LEG_07_45,LEG_07_177"
  },
  {
    "id": 269,
    "start_hour": 169,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0269",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_08_47,LEG_08_183,LEG_08_186"
  },
  {
    "id": 270,
    "start_hour": 146,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0270",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_07_171"
  },
  {
    "id": 271,
    "start_hour": 176,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0271",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_08_174"
  },
  {
    "id": 272,
    "start_hour": 147,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0272",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_07_105,LEG_07_127,LEG_07_26"
  },
  {
    "id": 273,
    "start_hour": 168,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0273",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_08_167,LEG_08_20,LEG_08_45"
  },
  {
    "id": 274,
    "start_hour": 152,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0274",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_07_54,LEG_07_49"
  },
  {
    "id": 275,
    "start_hour": 152,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0275",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_07_108,LEG_07_109"
  },
  {
    "id": 276,
    "start_hour": 145,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0276",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_07_178,LEG_07_176,LEG_07_160,LEG_07_80"
  },
  {
    "id": 277,
    "start_hour": 133,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0277",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_07_172,LEG_07_31"
  },
  {
    "id": 278,
    "start_hour": 158,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0278",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_08_90,LEG_08_93,LEG_08_4"
  },
  {
    "id": 279,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0279",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_09_139,LEG_09_183,LEG_09_186"
  },
  {
    "id": 280,
    "start_hour": 155,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0280",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_07_10"
  },
  {
    "id": 281,
    "start_hour": 172,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0281",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_08_12,LEG_08_51"
  },
  {
    "id": 282,
    "start_hour": 194,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0282",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_09_41,LEG_09_42,LEG_09_50"
  },
  {
    "id": 283,
    "start_hour": 153,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0283",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_07_135,LEG_07_137"
  },
  {
    "id": 284,
    "start_hour": 157,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0284",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_08_81"
  },
  {
    "id": 285,
    "start_hour": 192,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0285",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_09_78,LEG_09_79,LEG_09_108,LEG_09_185"
  },
  {
    "id": 286,
    "start_hour": 216,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0286",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_10_60,LEG_10_70,LEG_10_184"
  },
  {
    "id": 287,
    "start_hour": 15,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0287",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_02_3,LEG_02_23"
  },
  {
    "id": 288,
    "start_hour": 27,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0288",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_02_142,LEG_02_141"
  },
  {
    "id": 289,
    "start_hour": 29,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0289",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_02_82,LEG_02_83"
  },
  {
    "id": 290,
    "start_hour": 27,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0290",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_02_30,LEG_02_29"
  },
  {
    "id": 291,
    "start_hour": 29,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0291",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_02_113,LEG_02_110"
  },
  {
    "id": 292,
    "start_hour": 29,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0292",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_02_144,LEG_02_153"
  },
  {
    "id": 293,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0293",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_02_1,LEG_02_5"
  },
  {
    "id": 294,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0294",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_02_94,LEG_02_98"
  },
  {
    "id": 295,
    "start_hour": 24,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0295",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_02_164,LEG_02_176,LEG_02_46"
  },
  {
    "id": 296,
    "start_hour": 48,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0296",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_03_7"
  },
  {
    "id": 297,
    "start_hour": 33,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0297",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_02_91,LEG_02_69"
  },
  {
    "id": 298,
    "start_hour": 38,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0298",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_03_24"
  },
  {
    "id": 299,
    "start_hour": 35,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0299",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_02_16"
  },
  {
    "id": 300,
    "start_hour": 47,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0300",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_03_19,LEG_03_102,LEG_03_100"
  },
  {
    "id": 301,
    "start_hour": 12,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0301",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_02_122,LEG_02_112,LEG_02_95,LEG_02_168,LEG_02_111"
  },
  {
    "id": 302,
    "start_hour": 48,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0302",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_03_101,LEG_03_155,LEG_03_156"
  },
  {
    "id": 303,
    "start_hour": 29,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0303",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_02_6,LEG_02_2"
  },
  {
    "id": 304,
    "start_hour": 30,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0304",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_02_115,LEG_02_77"
  },
  {
    "id": 305,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0305",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_02_119,LEG_02_90"
  },
  {
    "id": 306,
    "start_hour": 24,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0306",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_02_14,LEG_02_36"
  },
  {
    "id": 307,
    "start_hour": 39,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0307",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_03_68,LEG_03_158"
  },
  {
    "id": 308,
    "start_hour": 61,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0308",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_04_173,LEG_04_31"
  },
  {
    "id": 309,
    "start_hour": 36,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0309",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_02_21"
  },
  {
    "id": 310,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0310",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_03_144,LEG_03_114,LEG_03_111,LEG_03_112"
  },
  {
    "id": 311,
    "start_hour": 72,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0311",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_04_101,LEG_04_155,LEG_04_156"
  },
  {
    "id": 312,
    "start_hour": 36,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0312",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_02_4"
  },
  {
    "id": 313,
    "start_hour": 50,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0313",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_03_139,LEG_03_79,LEG_03_108,LEG_03_46"
  },
  {
    "id": 314,
    "start_hour": 72,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0314",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_04_7"
  },
  {
    "id": 315,
    "start_hour": 34,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0315",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_02_97"
  },
  {
    "id": 316,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0316",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_03_97,LEG_03_150"
  },
  {
    "id": 317,
    "start_hour": 74,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0317",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_04_148,LEG_04_189"
  },
  {
    "id": 318,
    "start_hour": 96,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0318",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_05_184"
  },
  {
    "id": 319,
    "start_hour": 36,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0319",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_02_152"
  },
  {
    "id": 320,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0320",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_03_32,LEG_03_170"
  },
  {
    "id": 321,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0321",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_04_113,LEG_04_16"
  },
  {
    "id": 322,
    "start_hour": 95,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0322",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_05_20,LEG_05_162,LEG_05_81"
  },
  {
    "id": 323,
    "start_hour": 36,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0323",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_02_28"
  },
  {
    "id": 324,
    "start_hour": 48,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0324",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_03_82,LEG_03_121"
  },
  {
    "id": 325,
    "start_hour": 72,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0325",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_04_129,LEG_04_131"
  },
  {
    "id": 326,
    "start_hour": 97,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0326",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_05_132,LEG_05_94,LEG_05_96"
  },
  {
    "id": 327,
    "start_hour": 36,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0327",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_02_189"
  },
  {
    "id": 328,
    "start_hour": 58,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0328",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_03_192"
  },
  {
    "id": 329,
    "start_hour": 62,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0329",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_04_117,LEG_04_115,LEG_04_76,LEG_04_134"
  },
  {
    "id": 330,
    "start_hour": 32,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0330",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_02_150,LEG_02_145"
  },
  {
    "id": 331,
    "start_hour": 38,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0331",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_03_135,LEG_03_157"
  },
  {
    "id": 332,
    "start_hour": 60,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0332",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_04_123,LEG_04_164,LEG_04_127"
  },
  {
    "id": 333,
    "start_hour": 98,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0333",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_05_118,LEG_05_19,LEG_05_18"
  },
  {
    "id": 334,
    "start_hour": 34,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0334",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_02_126"
  },
  {
    "id": 335,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0335",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_03_119,LEG_03_127"
  },
  {
    "id": 336,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0336",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_04_119,LEG_04_150"
  },
  {
    "id": 337,
    "start_hour": 98,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0337",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_05_144,LEG_05_115,LEG_05_73"
  },
  {
    "id": 338,
    "start_hour": 470,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0338",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_21_36,LEG_21_106"
  },
  {
    "id": 339,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0339",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_21_63,LEG_21_111,LEG_21_11"
  },
  {
    "id": 340,
    "start_hour": 503,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0340",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_22_165,LEG_22_41,LEG_22_64"
  },
  {
    "id": 341,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0341",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_21_39,LEG_21_37"
  },
  {
    "id": 342,
    "start_hour": 495,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0342",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_22_70"
  },
  {
    "id": 343,
    "start_hour": 532,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0343",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_23_180,LEG_23_40"
  },
  {
    "id": 344,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0344",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_21_67,LEG_21_68"
  },
  {
    "id": 345,
    "start_hour": 494,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0345",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_22_69,LEG_22_66,LEG_22_39,LEG_22_37"
  },
  {
    "id": 346,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0346",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_21_105,LEG_21_88,LEG_21_89"
  },
  {
    "id": 347,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0347",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_22_91,LEG_22_131"
  },
  {
    "id": 348,
    "start_hour": 530,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0348",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_23_123,LEG_23_145"
  },
  {
    "id": 349,
    "start_hour": 552,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0349",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_24_126,LEG_24_134,LEG_24_74"
  },
  {
    "id": 350,
    "start_hour": 471,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0350",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_21_70,LEG_21_162"
  },
  {
    "id": 351,
    "start_hour": 493,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0351",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_22_176,LEG_22_32"
  },
  {
    "id": 352,
    "start_hour": 518,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0352",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_23_151,LEG_23_156,LEG_23_28,LEG_23_60"
  },
  {
    "id": 353,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0353",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_21_65,LEG_21_122"
  },
  {
    "id": 354,
    "start_hour": 494,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0354",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_22_129,LEG_22_128,LEG_22_193"
  },
  {
    "id": 355,
    "start_hour": 538,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0355",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_23_196"
  },
  {
    "id": 356,
    "start_hour": 543,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0356",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_24_33,LEG_24_61,LEG_24_39,LEG_24_37"
  },
  {
    "id": 357,
    "start_hour": 518,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0357",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_23_36,LEG_23_106"
  },
  {
    "id": 358,
    "start_hour": 527,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0358",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_23_24,LEG_23_134,LEG_23_74"
  },
  {
    "id": 359,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0359",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_23_67,LEG_23_68"
  },
  {
    "id": 360,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0360",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_23_63,LEG_23_111,LEG_23_11"
  },
  {
    "id": 361,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0361",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_24_166,LEG_24_176"
  },
  {
    "id": 362,
    "start_hour": 584,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0362",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_25_178"
  },
  {
    "id": 363,
    "start_hour": 590,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0363",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_26_176,LEG_26_172,LEG_26_167,LEG_26_37"
  },
  {
    "id": 364,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0364",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_23_72,LEG_23_188,LEG_23_182"
  },
  {
    "id": 365,
    "start_hour": 553,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0365",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_24_49,LEG_24_81,LEG_24_112,LEG_24_48"
  },
  {
    "id": 366,
    "start_hour": 576,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0366",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_25_7,LEG_25_193"
  },
  {
    "id": 367,
    "start_hour": 600,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0367",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_26_180,LEG_26_128,LEG_26_71"
  },
  {
    "id": 368,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0368",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_23_65,LEG_23_122"
  },
  {
    "id": 369,
    "start_hour": 542,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0369",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_24_139,LEG_24_161,LEG_24_194"
  },
  {
    "id": 370,
    "start_hour": 586,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0370",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_25_196"
  },
  {
    "id": 371,
    "start_hour": 590,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0371",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_26_142,LEG_26_146,LEG_26_24,LEG_26_58"
  },
  {
    "id": 372,
    "start_hour": 422,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0372",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_19_32,LEG_19_101"
  },
  {
    "id": 373,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0373",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_19_59,LEG_19_108,LEG_19_11"
  },
  {
    "id": 374,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0374",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_20_173,LEG_20_167,LEG_20_37"
  },
  {
    "id": 375,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0375",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_19_60,LEG_19_119"
  },
  {
    "id": 376,
    "start_hour": 446,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0376",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_20_141,LEG_20_146,LEG_20_136"
  },
  {
    "id": 377,
    "start_hour": 480,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0377",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_21_126,LEG_21_134,LEG_21_74"
  },
  {
    "id": 378,
    "start_hour": 438,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0378",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_19_71,LEG_19_21,LEG_19_183"
  },
  {
    "id": 379,
    "start_hour": 459,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0379",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_20_156"
  },
  {
    "id": 380,
    "start_hour": 468,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0380",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_21_13,LEG_21_0,LEG_21_172,LEG_21_59"
  },
  {
    "id": 381,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0381",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_19_67,LEG_19_180,LEG_19_52"
  },
  {
    "id": 382,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0382",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_20_42,LEG_20_131,LEG_20_133"
  },
  {
    "id": 383,
    "start_hour": 469,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0383",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_21_83"
  },
  {
    "id": 384,
    "start_hour": 504,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0384",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_22_80,LEG_22_179,LEG_22_40"
  },
  {
    "id": 385,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0385",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_25_97,LEG_25_101"
  },
  {
    "id": 386,
    "start_hour": 567,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0386",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_25_3,LEG_25_24"
  },
  {
    "id": 387,
    "start_hour": 576,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0387",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_25_169,LEG_25_170"
  },
  {
    "id": 388,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0388",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_25_1,LEG_25_5"
  },
  {
    "id": 389,
    "start_hour": 581,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0389",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_25_149,LEG_25_158,LEG_25_29"
  },
  {
    "id": 390,
    "start_hour": 600,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0390",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_26_78,LEG_26_112,LEG_26_110"
  },
  {
    "id": 391,
    "start_hour": 584,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0391",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_25_155,LEG_25_150"
  },
  {
    "id": 392,
    "start_hour": 590,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0392",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_26_123,LEG_26_122"
  },
  {
    "id": 393,
    "start_hour": 583,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0393",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_25_19,LEG_25_18"
  },
  {
    "id": 394,
    "start_hour": 589,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0394",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_26_9,LEG_26_101,LEG_26_102,LEG_26_126"
  },
  {
    "id": 395,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0395",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_25_90,LEG_25_130"
  },
  {
    "id": 396,
    "start_hour": 590,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0396",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_26_133,LEG_26_151"
  },
  {
    "id": 397,
    "start_hour": 580,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0397",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_25_136,LEG_25_26,LEG_25_27"
  },
  {
    "id": 398,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0398",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_26_158,LEG_26_27,LEG_26_26"
  },
  {
    "id": 399,
    "start_hour": 585,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0399",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_25_94,LEG_25_71"
  },
  {
    "id": 400,
    "start_hour": 590,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0400",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_26_23"
  },
  {
    "id": 401,
    "start_hour": 579,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0401",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_25_147,LEG_25_146"
  },
  {
    "id": 402,
    "start_hour": 588,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0402",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_26_81,LEG_26_90,LEG_26_137"
  },
  {
    "id": 403,
    "start_hour": 576,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0403",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_25_15,LEG_25_38,LEG_25_75"
  },
  {
    "id": 404,
    "start_hour": 599,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0404",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_26_70,LEG_26_32,LEG_26_144"
  },
  {
    "id": 405,
    "start_hour": 581,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0405",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_25_6,LEG_25_2"
  },
  {
    "id": 406,
    "start_hour": 579,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0406",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_25_21,LEG_25_47,LEG_25_48"
  },
  {
    "id": 407,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0407",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_26_135,LEG_26_173,LEG_26_183"
  },
  {
    "id": 408,
    "start_hour": 624,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0408",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_27_16,LEG_27_27"
  },
  {
    "id": 409,
    "start_hour": 582,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0409",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_25_120,LEG_25_79"
  },
  {
    "id": 410,
    "start_hour": 590,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0410",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_26_115,LEG_26_113,LEG_26_75,LEG_26_132"
  },
  {
    "id": 411,
    "start_hour": 586,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0411",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_25_100"
  },
  {
    "id": 412,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0412",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_26_92,LEG_26_161"
  },
  {
    "id": 413,
    "start_hour": 626,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0413",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_27_115,LEG_27_77"
  },
  {
    "id": 414,
    "start_hour": 649,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0414",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_28_125"
  },
  {
    "id": 415,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0415",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_25_124,LEG_25_93"
  },
  {
    "id": 416,
    "start_hour": 591,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0416",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_26_29,LEG_26_105,LEG_26_39,LEG_26_16"
  },
  {
    "id": 417,
    "start_hour": 613,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D0417",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_27_29,LEG_27_4,LEG_27_111"
  },
  {
    "id": 418,
    "start_hour": 651,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0418",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_28_133"
  },
  {
    "id": 419,
    "start_hour": 513,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0419",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_22_140,LEG_22_142"
  },
  {
    "id": 420,
    "start_hour": 517,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0420",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_23_83"
  },
  {
    "id": 421,
    "start_hour": 552,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0421",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_24_80,LEG_24_144,LEG_24_57,LEG_24_182"
  },
  {
    "id": 422,
    "start_hour": 577,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0422",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_25_49,LEG_25_50,LEG_25_55"
  },
  {
    "id": 423,
    "start_hour": 515,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0423",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_22_10"
  },
  {
    "id": 424,
    "start_hour": 532,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0424",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_23_12"
  },
  {
    "id": 425,
    "start_hour": 542,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0425",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_24_191,LEG_24_186,LEG_24_50,LEG_24_55"
  },
  {
    "id": 426,
    "start_hour": 507,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0426",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_22_108,LEG_22_132,LEG_22_29"
  },
  {
    "id": 427,
    "start_hour": 528,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0427",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_23_84,LEG_23_175"
  },
  {
    "id": 428,
    "start_hour": 552,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0428",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_24_179,LEG_24_44,LEG_24_52"
  },
  {
    "id": 429,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0429",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_22_45,LEG_22_16,LEG_22_173"
  },
  {
    "id": 430,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0430",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_23_185,LEG_23_184,LEG_23_9"
  },
  {
    "id": 431,
    "start_hour": 509,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0431",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_22_50,LEG_22_55"
  },
  {
    "id": 432,
    "start_hour": 518,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0432",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_23_191,LEG_23_186,LEG_23_50,LEG_23_55"
  },
  {
    "id": 433,
    "start_hour": 506,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0433",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_22_175"
  },
  {
    "id": 434,
    "start_hour": 536,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0434",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_23_178"
  },
  {
    "id": 435,
    "start_hour": 505,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0435",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_22_166,LEG_22_163"
  },
  {
    "id": 436,
    "start_hour": 505,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0436",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_22_182,LEG_22_180,LEG_22_164,LEG_22_82"
  },
  {
    "id": 437,
    "start_hour": 512,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0437",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_22_113,LEG_22_114"
  },
  {
    "id": 438,
    "start_hour": 493,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0438",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_22_141,LEG_22_42"
  },
  {
    "id": 439,
    "start_hour": 512,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0439",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_22_177"
  },
  {
    "id": 440,
    "start_hour": 493,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0440",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_22_54,LEG_22_46"
  },
  {
    "id": 441,
    "start_hour": 512,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0441",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_22_56,LEG_22_51"
  },
  {
    "id": 442,
    "start_hour": 566,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0442",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_25_36,LEG_25_106"
  },
  {
    "id": 443,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0443",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_25_105,LEG_25_88,LEG_25_89"
  },
  {
    "id": 444,
    "start_hour": 602,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0444",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_26_85,LEG_26_91,LEG_26_160,LEG_26_136"
  },
  {
    "id": 445,
    "start_hour": 625,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0445",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_27_87,LEG_27_102,LEG_27_59"
  },
  {
    "id": 446,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0446",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_25_39,LEG_25_37"
  },
  {
    "id": 447,
    "start_hour": 591,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0447",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_26_68"
  },
  {
    "id": 448,
    "start_hour": 628,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0448",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_27_13,LEG_27_122"
  },
  {
    "id": 449,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0449",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_25_65,LEG_25_122"
  },
  {
    "id": 450,
    "start_hour": 590,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0450",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_26_86,LEG_26_89"
  },
  {
    "id": 451,
    "start_hour": 612,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0451",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_27_92,LEG_27_134,LEG_27_68"
  },
  {
    "id": 452,
    "start_hour": 637,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0452",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_28_143"
  },
  {
    "id": 453,
    "start_hour": 582,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0453",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_25_67,LEG_25_68"
  },
  {
    "id": 454,
    "start_hour": 590,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0454",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_26_66,LEG_26_63,LEG_26_62,LEG_26_175"
  },
  {
    "id": 455,
    "start_hour": 613,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0455",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_27_141"
  },
  {
    "id": 456,
    "start_hour": 648,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0456",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_28_158,LEG_28_124,LEG_28_73"
  },
  {
    "id": 457,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0457",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_25_63,LEG_25_111,LEG_25_11"
  },
  {
    "id": 458,
    "start_hour": 599,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0458",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_26_153,LEG_26_163"
  },
  {
    "id": 459,
    "start_hour": 632,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0459",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_27_142"
  },
  {
    "id": 460,
    "start_hour": 638,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0460",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_28_174,LEG_28_58,LEG_28_190,LEG_28_204"
  },
  {
    "id": 461,
    "start_hour": 327,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0461",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_15_3,LEG_15_24"
  },
  {
    "id": 462,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0462",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_15_97,LEG_15_101"
  },
  {
    "id": 463,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0463",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_15_1,LEG_15_5"
  },
  {
    "id": 464,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0464",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_15_31,LEG_15_30,LEG_15_27"
  },
  {
    "id": 465,
    "start_hour": 360,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0465",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_16_171,LEG_16_21,LEG_16_16"
  },
  {
    "id": 466,
    "start_hour": 341,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0466",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_15_149,LEG_15_158,LEG_15_29"
  },
  {
    "id": 467,
    "start_hour": 360,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0467",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_16_84,LEG_16_118,LEG_16_115"
  },
  {
    "id": 468,
    "start_hour": 345,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0468",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_15_94,LEG_15_71"
  },
  {
    "id": 469,
    "start_hour": 350,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0469",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_16_25"
  },
  {
    "id": 470,
    "start_hour": 342,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0470",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_15_120,LEG_15_79"
  },
  {
    "id": 471,
    "start_hour": 349,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D0471",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_16_8,LEG_16_107,LEG_16_163,LEG_16_77"
  },
  {
    "id": 472,
    "start_hour": 336,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0472",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_15_169,LEG_15_66,LEG_15_65"
  },
  {
    "id": 473,
    "start_hour": 360,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0473",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_16_62,LEG_16_105,LEG_16_170"
  },
  {
    "id": 474,
    "start_hour": 326,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0474",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_15_121,LEG_15_119,LEG_15_78,LEG_15_138"
  },
  {
    "id": 475,
    "start_hour": 341,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0475",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_15_6,LEG_15_2"
  },
  {
    "id": 476,
    "start_hour": 326,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D0476",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_15_151,LEG_15_156"
  },
  {
    "id": 477,
    "start_hour": 348,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0477",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_16_127,LEG_16_185,LEG_16_110,LEG_16_14,LEG_16_116"
  },
  {
    "id": 478,
    "start_hour": 384,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0478",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_17_103,LEG_17_159,LEG_17_160"
  },
  {
    "id": 479,
    "start_hour": 326,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0479",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_15_129,LEG_15_128,LEG_15_157"
  },
  {
    "id": 480,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0480",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_16_34,LEG_16_98,LEG_16_173,LEG_16_17"
  },
  {
    "id": 481,
    "start_hour": 383,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0481",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_17_20,LEG_17_104,LEG_17_102"
  },
  {
    "id": 482,
    "start_hour": 324,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0482",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_15_127,LEG_15_170,LEG_15_154"
  },
  {
    "id": 483,
    "start_hour": 362,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0483",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_16_152,LEG_16_193"
  },
  {
    "id": 484,
    "start_hour": 384,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0484",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_17_195"
  },
  {
    "id": 485,
    "start_hour": 344,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0485",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_15_155,LEG_15_150"
  },
  {
    "id": 486,
    "start_hour": 350,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D0486",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_16_92,LEG_16_95"
  },
  {
    "id": 487,
    "start_hour": 372,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0487",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_17_127,LEG_17_117,LEG_17_118,LEG_17_115"
  },
  {
    "id": 488,
    "start_hour": 336,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0488",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_15_15,LEG_15_38"
  },
  {
    "id": 489,
    "start_hour": 351,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0489",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_16_70,LEG_16_162"
  },
  {
    "id": 490,
    "start_hour": 373,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0490",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_17_177,LEG_17_32"
  },
  {
    "id": 491,
    "start_hour": 343,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0491",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_15_19,LEG_15_18"
  },
  {
    "id": 492,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0492",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_16_129,LEG_16_128,LEG_16_157"
  },
  {
    "id": 493,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0493",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_17_34,LEG_17_98,LEG_17_173,LEG_17_116"
  },
  {
    "id": 494,
    "start_hour": 408,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0494",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_18_103,LEG_18_159,LEG_18_160"
  },
  {
    "id": 495,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0495",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_15_124,LEG_15_93"
  },
  {
    "id": 496,
    "start_hour": 351,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0496",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_16_33,LEG_16_61,LEG_16_58"
  },
  {
    "id": 497,
    "start_hour": 387,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0497",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_17_61,LEG_17_75"
  },
  {
    "id": 498,
    "start_hour": 407,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0498",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_18_73,LEG_18_35,LEG_18_153"
  },
  {
    "id": 499,
    "start_hour": 346,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0499",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_15_100"
  },
  {
    "id": 500,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0500",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_16_99,LEG_16_154"
  },
  {
    "id": 501,
    "start_hour": 386,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0501",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_17_152,LEG_17_193"
  },
  {
    "id": 502,
    "start_hour": 408,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0502",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_18_195"
  },
  {
    "id": 503,
    "start_hour": 341,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0503",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_15_85,LEG_15_86,LEG_15_145"
  },
  {
    "id": 504,
    "start_hour": 360,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0504",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_16_126,LEG_16_125"
  },
  {
    "id": 505,
    "start_hour": 384,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0505",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_17_133,LEG_17_125"
  },
  {
    "id": 506,
    "start_hour": 408,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0506",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_18_133"
  },
  {
    "id": 507,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0507",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_15_90,LEG_15_130"
  },
  {
    "id": 508,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0508",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_16_139,LEG_16_161,LEG_16_194"
  },
  {
    "id": 509,
    "start_hour": 394,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0509",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_17_196"
  },
  {
    "id": 510,
    "start_hour": 539,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0510",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_23_10"
  },
  {
    "id": 511,
    "start_hour": 556,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0511",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_24_12,LEG_24_53"
  },
  {
    "id": 512,
    "start_hour": 578,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0512",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_25_43,LEG_25_182"
  },
  {
    "id": 513,
    "start_hour": 601,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0513",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_26_48,LEG_26_49,LEG_26_52"
  },
  {
    "id": 514,
    "start_hour": 517,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0514",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_23_177,LEG_23_32"
  },
  {
    "id": 515,
    "start_hour": 542,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0515",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_24_92,LEG_24_95,LEG_24_4"
  },
  {
    "id": 516,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0516",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_25_143,LEG_25_187,LEG_25_190"
  },
  {
    "id": 517,
    "start_hour": 539,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0517",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_23_53"
  },
  {
    "id": 518,
    "start_hour": 554,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0518",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_24_43,LEG_24_189"
  },
  {
    "id": 519,
    "start_hour": 576,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0519",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_25_179,LEG_25_144,LEG_25_57,LEG_25_165,LEG_25_82"
  },
  {
    "id": 520,
    "start_hour": 537,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0520",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_23_140,LEG_23_142"
  },
  {
    "id": 521,
    "start_hour": 541,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0521",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_24_83"
  },
  {
    "id": 522,
    "start_hour": 576,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0522",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_25_80,LEG_25_81,LEG_25_112"
  },
  {
    "id": 523,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0523",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_23_183,LEG_23_59,LEG_23_58"
  },
  {
    "id": 524,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0524",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_24_109,LEG_24_184,LEG_24_9"
  },
  {
    "id": 525,
    "start_hour": 531,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0525",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_23_108,LEG_23_132,LEG_23_27"
  },
  {
    "id": 526,
    "start_hour": 552,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0526",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_24_171,LEG_24_21,LEG_24_47"
  },
  {
    "id": 527,
    "start_hour": 529,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0527",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_23_41,LEG_23_64,LEG_23_75"
  },
  {
    "id": 528,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0528",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_24_73,LEG_24_72,LEG_24_188"
  },
  {
    "id": 529,
    "start_hour": 536,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0529",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_23_113,LEG_23_114"
  },
  {
    "id": 530,
    "start_hour": 529,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0530",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_23_45,LEG_23_47"
  },
  {
    "id": 531,
    "start_hour": 517,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0531",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_23_54,LEG_23_46"
  },
  {
    "id": 532,
    "start_hour": 529,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0532",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_23_167,LEG_23_164"
  },
  {
    "id": 533,
    "start_hour": 517,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0533",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_23_141,LEG_23_42"
  },
  {
    "id": 534,
    "start_hour": 536,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0534",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_23_56,LEG_23_51"
  },
  {
    "id": 535,
    "start_hour": 254,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0535",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_12_62,LEG_12_56"
  },
  {
    "id": 536,
    "start_hour": 254,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0536",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_12_31,LEG_12_97"
  },
  {
    "id": 537,
    "start_hour": 266,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0537",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_12_65,LEG_12_172,LEG_12_37"
  },
  {
    "id": 538,
    "start_hour": 276,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0538",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_13_160"
  },
  {
    "id": 539,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0539",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_12_60,LEG_12_61"
  },
  {
    "id": 540,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0540",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_12_34,LEG_12_32"
  },
  {
    "id": 541,
    "start_hour": 279,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0541",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_13_60"
  },
  {
    "id": 542,
    "start_hour": 316,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0542",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_14_174,LEG_14_39"
  },
  {
    "id": 543,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0543",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_12_57,LEG_12_102,LEG_12_11"
  },
  {
    "id": 544,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0544",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_13_170,LEG_13_73,LEG_13_101,LEG_13_173"
  },
  {
    "id": 545,
    "start_hour": 312,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0545",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_14_173,LEG_14_81,LEG_14_108,LEG_14_176"
  },
  {
    "id": 546,
    "start_hour": 337,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0546",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_15_49,LEG_15_187,LEG_15_59"
  },
  {
    "id": 547,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0547",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_12_95,LEG_12_165,LEG_12_50"
  },
  {
    "id": 548,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0548",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_13_43,LEG_13_129,LEG_13_131"
  },
  {
    "id": 549,
    "start_hour": 301,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0549",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_14_83"
  },
  {
    "id": 550,
    "start_hour": 336,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0550",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_15_80,LEG_15_180,LEG_15_40"
  },
  {
    "id": 551,
    "start_hour": 446,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0551",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_20_33,LEG_20_99"
  },
  {
    "id": 552,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0552",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_20_56,LEG_20_102,LEG_20_10"
  },
  {
    "id": 553,
    "start_hour": 479,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0553",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_21_166,LEG_21_41,LEG_21_64"
  },
  {
    "id": 554,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0554",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_20_59,LEG_20_60"
  },
  {
    "id": 555,
    "start_hour": 458,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0555",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_20_97,LEG_20_168,LEG_20_43"
  },
  {
    "id": 556,
    "start_hour": 480,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0556",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_21_7,LEG_21_193"
  },
  {
    "id": 557,
    "start_hour": 504,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0557",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_22_194,LEG_22_134,LEG_22_74"
  },
  {
    "id": 558,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0558",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_20_57,LEG_20_113"
  },
  {
    "id": 559,
    "start_hour": 470,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0559",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_21_139,LEG_21_161,LEG_21_194"
  },
  {
    "id": 560,
    "start_hour": 514,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0560",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_22_195"
  },
  {
    "id": 561,
    "start_hour": 519,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0561",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_23_3"
  },
  {
    "id": 562,
    "start_hour": 462,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0562",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_20_67,LEG_20_20,LEG_20_177"
  },
  {
    "id": 563,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0563",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_21_168,LEG_21_131"
  },
  {
    "id": 564,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0564",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_22_123,LEG_22_28,LEG_22_60"
  },
  {
    "id": 565,
    "start_hour": 648,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0565",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_28_11,LEG_28_9,LEG_28_179"
  },
  {
    "id": 566,
    "start_hour": 672,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0566",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_29_93,LEG_29_135"
  },
  {
    "id": 567,
    "start_hour": 699,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0567",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_30_137,LEG_30_87,LEG_30_76"
  },
  {
    "id": 568,
    "start_hour": 657,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0568",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_28_17"
  },
  {
    "id": 569,
    "start_hour": 690,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0569",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_30_16,LEG_30_11,LEG_30_9"
  },
  {
    "id": 570,
    "start_hour": 658,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0570",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_28_153"
  },
  {
    "id": 571,
    "start_hour": 661,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0571",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_29_166,LEG_29_200,LEG_29_170,LEG_29_22"
  },
  {
    "id": 572,
    "start_hour": 685,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0572",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_30_48"
  },
  {
    "id": 573,
    "start_hour": 652,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0573",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_28_154,LEG_28_122,LEG_28_170"
  },
  {
    "id": 574,
    "start_hour": 674,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0574",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_29_141,LEG_29_31,LEG_29_109"
  },
  {
    "id": 575,
    "start_hour": 653,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0575",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_28_10,LEG_28_52,LEG_28_104"
  },
  {
    "id": 576,
    "start_hour": 660,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0576",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_29_110"
  },
  {
    "id": 577,
    "start_hour": 651,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0577",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_28_173"
  },
  {
    "id": 578,
    "start_hour": 680,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0578",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_29_179"
  },
  {
    "id": 579,
    "start_hour": 638,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0579",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_28_162,LEG_28_98"
  },
  {
    "id": 580,
    "start_hour": 664,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0580",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_29_165,LEG_29_160"
  },
  {
    "id": 581,
    "start_hour": 657,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0581",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_28_53"
  },
  {
    "id": 582,
    "start_hour": 661,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0582",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_29_49"
  },
  {
    "id": 583,
    "start_hour": 648,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0583",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_28_15,LEG_28_29,LEG_28_83"
  },
  {
    "id": 584,
    "start_hour": 673,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0584",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_29_42,LEG_29_150,LEG_29_12"
  },
  {
    "id": 585,
    "start_hour": 650,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0585",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_28_127,LEG_28_96"
  },
  {
    "id": 586,
    "start_hour": 650,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0586",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_28_99,LEG_28_51"
  },
  {
    "id": 587,
    "start_hour": 638,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0587",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_28_106,LEG_28_100"
  },
  {
    "id": 588,
    "start_hour": 638,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0588",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_28_182,LEG_28_172"
  },
  {
    "id": 589,
    "start_hour": 638,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0589",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_28_75,LEG_28_186"
  },
  {
    "id": 590,
    "start_hour": 636,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0590",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_28_61,LEG_28_38,LEG_28_36,LEG_28_30,LEG_28_107"
  },
  {
    "id": 591,
    "start_hour": 648,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0591",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_28_56,LEG_28_54"
  },
  {
    "id": 592,
    "start_hour": 590,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0592",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_26_33,LEG_26_100"
  },
  {
    "id": 593,
    "start_hour": 606,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0593",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_26_64,LEG_26_65"
  },
  {
    "id": 594,
    "start_hour": 606,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0594",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_26_73,LEG_26_22,LEG_26_177"
  },
  {
    "id": 595,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0595",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_27_151,LEG_27_82"
  },
  {
    "id": 596,
    "start_hour": 636,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0596",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_28_71"
  },
  {
    "id": 597,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0597",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_26_98,LEG_26_57,LEG_26_56"
  },
  {
    "id": 598,
    "start_hour": 628,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0598",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_27_55,LEG_27_42,LEG_27_40"
  },
  {
    "id": 599,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0599",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_26_61,LEG_26_107,LEG_26_11"
  },
  {
    "id": 600,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0600",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_27_153,LEG_27_144,LEG_27_14,LEG_27_143"
  },
  {
    "id": 601,
    "start_hour": 649,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0601",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_28_178,LEG_28_57"
  },
  {
    "id": 602,
    "start_hour": 672,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0602",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_29_103,LEG_29_14,LEG_29_152"
  },
  {
    "id": 603,
    "start_hour": 296,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0603",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_13_163"
  },
  {
    "id": 604,
    "start_hour": 277,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0604",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_13_48,LEG_13_42"
  },
  {
    "id": 605,
    "start_hour": 277,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0605",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_13_130,LEG_13_40"
  },
  {
    "id": 606,
    "start_hour": 291,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0606",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_13_99,LEG_13_121,LEG_13_19"
  },
  {
    "id": 607,
    "start_hour": 314,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0607",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_14_42,LEG_14_178,LEG_14_9"
  },
  {
    "id": 608,
    "start_hour": 289,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0608",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_13_152,LEG_13_150,LEG_13_166"
  },
  {
    "id": 609,
    "start_hour": 313,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0609",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_14_48,LEG_14_49,LEG_14_54"
  },
  {
    "id": 610,
    "start_hour": 292,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0610",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_13_164,LEG_13_39,LEG_13_65"
  },
  {
    "id": 611,
    "start_hour": 311,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0611",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_14_72,LEG_14_62,LEG_14_107"
  },
  {
    "id": 612,
    "start_hour": 287,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0612",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_13_12,LEG_13_66,LEG_13_171"
  },
  {
    "id": 613,
    "start_hour": 312,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0613",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_14_61,LEG_14_71,LEG_14_182"
  },
  {
    "id": 614,
    "start_hour": 289,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0614",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_13_167,LEG_13_165,LEG_13_151,LEG_13_74"
  },
  {
    "id": 615,
    "start_hour": 296,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0615",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_13_102,LEG_13_103"
  },
  {
    "id": 616,
    "start_hour": 296,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0616",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_13_50,LEG_13_47"
  },
  {
    "id": 617,
    "start_hour": 299,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0617",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_13_9"
  },
  {
    "id": 618,
    "start_hour": 316,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0618",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_14_12"
  },
  {
    "id": 619,
    "start_hour": 326,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0619",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_15_191,LEG_15_186,LEG_15_44,LEG_15_52"
  },
  {
    "id": 620,
    "start_hour": 277,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0620",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_13_162,LEG_13_32"
  },
  {
    "id": 621,
    "start_hour": 302,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D0621",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_14_92,LEG_14_95"
  },
  {
    "id": 622,
    "start_hour": 324,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0622",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_15_13,LEG_15_0,LEG_15_172,LEG_15_190"
  },
  {
    "id": 623,
    "start_hour": 50,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0623",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_03_103,LEG_03_86,LEG_03_87"
  },
  {
    "id": 624,
    "start_hour": 74,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0624",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_04_89,LEG_04_98"
  },
  {
    "id": 625,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0625",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_05_91,LEG_05_87"
  },
  {
    "id": 626,
    "start_hour": 124,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0626",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_06_34,LEG_06_35,LEG_06_33"
  },
  {
    "id": 627,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0627",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_03_63,LEG_03_118"
  },
  {
    "id": 628,
    "start_hour": 62,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0628",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_04_147,LEG_04_152,LEG_04_153"
  },
  {
    "id": 629,
    "start_hour": 97,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0629",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_05_29,LEG_05_182"
  },
  {
    "id": 630,
    "start_hour": 120,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0630",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_06_173,LEG_06_117,LEG_06_63"
  },
  {
    "id": 631,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0631",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_03_33,LEG_03_149"
  },
  {
    "id": 632,
    "start_hour": 60,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0632",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_04_13,LEG_04_0,LEG_04_20,LEG_04_45,LEG_04_11"
  },
  {
    "id": 633,
    "start_hour": 95,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0633",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_05_156,LEG_05_173,LEG_05_54"
  },
  {
    "id": 634,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0634",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_03_65,LEG_03_66"
  },
  {
    "id": 635,
    "start_hour": 62,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0635",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_04_67,LEG_04_64,LEG_04_37,LEG_04_35"
  },
  {
    "id": 636,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0636",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_03_61,LEG_03_107,LEG_03_11"
  },
  {
    "id": 637,
    "start_hour": 71,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0637",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_04_162,LEG_04_39,LEG_04_62"
  },
  {
    "id": 638,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0638",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_03_37,LEG_03_35"
  },
  {
    "id": 639,
    "start_hour": 38,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0639",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_03_67,LEG_03_60"
  },
  {
    "id": 640,
    "start_hour": 38,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0640",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_03_34,LEG_03_104"
  },
  {
    "id": 641,
    "start_hour": 533,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0641",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_23_118,LEG_23_115"
  },
  {
    "id": 642,
    "start_hour": 531,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0642",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_23_147,LEG_23_146"
  },
  {
    "id": 643,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0643",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_23_97,LEG_23_101"
  },
  {
    "id": 644,
    "start_hour": 530,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0644",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_23_1,LEG_23_5"
  },
  {
    "id": 645,
    "start_hour": 533,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0645",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_23_149,LEG_23_158"
  },
  {
    "id": 646,
    "start_hour": 531,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0646",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_23_31,LEG_23_30"
  },
  {
    "id": 647,
    "start_hour": 537,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0647",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_23_94,LEG_23_71"
  },
  {
    "id": 648,
    "start_hour": 542,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0648",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_24_25"
  },
  {
    "id": 649,
    "start_hour": 539,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0649",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_23_17"
  },
  {
    "id": 650,
    "start_hour": 551,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0650",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_24_20,LEG_24_104,LEG_24_102"
  },
  {
    "id": 651,
    "start_hour": 533,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0651",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_23_6,LEG_23_2"
  },
  {
    "id": 652,
    "start_hour": 534,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0652",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_23_120,LEG_23_79"
  },
  {
    "id": 653,
    "start_hour": 535,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0653",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_23_78,LEG_23_138"
  },
  {
    "id": 654,
    "start_hour": 534,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0654",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_23_124,LEG_23_93"
  },
  {
    "id": 655,
    "start_hour": 535,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0655",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_23_19,LEG_23_18"
  },
  {
    "id": 656,
    "start_hour": 540,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0656",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_23_157"
  },
  {
    "id": 657,
    "start_hour": 553,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0657",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_24_34,LEG_24_125"
  },
  {
    "id": 658,
    "start_hour": 576,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0658",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_25_133,LEG_25_118,LEG_25_115"
  },
  {
    "id": 659,
    "start_hour": 528,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0659",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_23_15,LEG_23_38"
  },
  {
    "id": 660,
    "start_hour": 543,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0660",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_24_70,LEG_24_162"
  },
  {
    "id": 661,
    "start_hour": 565,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0661",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_25_177,LEG_25_32"
  },
  {
    "id": 662,
    "start_hour": 528,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0662",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_23_169,LEG_23_181,LEG_23_48"
  },
  {
    "id": 663,
    "start_hour": 552,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0663",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_24_7,LEG_24_175"
  },
  {
    "id": 664,
    "start_hour": 576,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0664",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_25_62,LEG_25_35,LEG_25_153"
  },
  {
    "id": 665,
    "start_hour": 540,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0665",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_23_22"
  },
  {
    "id": 666,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0666",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_24_148,LEG_24_118,LEG_24_115,LEG_24_116"
  },
  {
    "id": 667,
    "start_hour": 576,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0667",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_25_103,LEG_25_159,LEG_25_160"
  },
  {
    "id": 668,
    "start_hour": 538,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0668",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_23_100"
  },
  {
    "id": 669,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0669",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_24_99,LEG_24_131"
  },
  {
    "id": 670,
    "start_hour": 578,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0670",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_25_123,LEG_25_145"
  },
  {
    "id": 671,
    "start_hour": 600,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0671",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_26_120,LEG_26_119"
  },
  {
    "id": 672,
    "start_hour": 538,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0672",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_23_131"
  },
  {
    "id": 673,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0673",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_24_123,LEG_24_89"
  },
  {
    "id": 674,
    "start_hour": 578,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0674",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_25_91"
  },
  {
    "id": 675,
    "start_hour": 588,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0675",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_26_14,LEG_26_0,LEG_26_159,LEG_26_82"
  },
  {
    "id": 676,
    "start_hour": 540,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0676",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_23_29"
  },
  {
    "id": 677,
    "start_hour": 552,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0677",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_24_84,LEG_24_135"
  },
  {
    "id": 678,
    "start_hour": 577,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0678",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_25_137,LEG_25_135"
  },
  {
    "id": 679,
    "start_hour": 601,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0679",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_26_131,LEG_26_95,LEG_26_97"
  },
  {
    "id": 680,
    "start_hour": 540,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0680",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_23_194"
  },
  {
    "id": 681,
    "start_hour": 562,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0681",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_24_196"
  },
  {
    "id": 682,
    "start_hour": 566,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0682",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_25_121,LEG_25_119,LEG_25_78,LEG_25_138"
  },
  {
    "id": 683,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0683",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_02_61,LEG_02_106,LEG_02_11"
  },
  {
    "id": 684,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0684",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_03_162,LEG_03_172"
  },
  {
    "id": 685,
    "start_hour": 80,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0685",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_04_174"
  },
  {
    "id": 686,
    "start_hour": 86,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0686",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_05_180,LEG_05_176,LEG_05_170,LEG_05_35"
  },
  {
    "id": 687,
    "start_hour": 26,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0687",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_02_33,LEG_02_148"
  },
  {
    "id": 688,
    "start_hour": 36,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0688",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_03_85,LEG_03_94,LEG_03_168,LEG_03_57"
  },
  {
    "id": 689,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0689",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_02_37,LEG_02_35"
  },
  {
    "id": 690,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0690",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_02_65,LEG_02_66"
  },
  {
    "id": 691,
    "start_hour": 14,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0691",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_02_34,LEG_02_103"
  },
  {
    "id": 692,
    "start_hour": 193,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0692",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_09_163,LEG_09_160"
  },
  {
    "id": 693,
    "start_hour": 181,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0693",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_09_137,LEG_09_40"
  },
  {
    "id": 694,
    "start_hour": 181,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0694",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_09_52,LEG_09_44"
  },
  {
    "id": 695,
    "start_hour": 203,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0695",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_09_10"
  },
  {
    "id": 696,
    "start_hour": 220,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0696",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_10_12"
  },
  {
    "id": 697,
    "start_hour": 195,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0697",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_09_106,LEG_09_128,LEG_09_4"
  },
  {
    "id": 698,
    "start_hour": 218,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0698",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_10_139,LEG_10_183,LEG_10_186"
  },
  {
    "id": 699,
    "start_hour": 200,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0699",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_09_54,LEG_09_49"
  },
  {
    "id": 700,
    "start_hour": 200,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0700",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_09_109,LEG_09_110"
  },
  {
    "id": 701,
    "start_hour": 194,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0701",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_09_172"
  },
  {
    "id": 702,
    "start_hour": 224,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0702",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_10_174"
  },
  {
    "id": 703,
    "start_hour": 230,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0703",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_11_187,LEG_11_182,LEG_11_48,LEG_11_53"
  },
  {
    "id": 704,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0704",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_09_179,LEG_09_57,LEG_09_56"
  },
  {
    "id": 705,
    "start_hour": 219,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0705",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_10_59,LEG_10_74,LEG_10_22,LEG_10_188"
  },
  {
    "id": 706,
    "start_hour": 243,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0706",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_11_164,LEG_11_170"
  },
  {
    "id": 707,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0707",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_12_169,LEG_12_168,LEG_12_9"
  },
  {
    "id": 708,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0708",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_09_43,LEG_09_15,LEG_09_170"
  },
  {
    "id": 709,
    "start_hour": 219,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0709",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_10_164,LEG_10_150"
  },
  {
    "id": 710,
    "start_hour": 242,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0710",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_11_148"
  },
  {
    "id": 711,
    "start_hour": 252,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D0711",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_12_14,LEG_12_0,LEG_12_125,LEG_12_101"
  },
  {
    "id": 712,
    "start_hour": 201,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0712",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_09_136,LEG_09_138"
  },
  {
    "id": 713,
    "start_hour": 205,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0713",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_10_81"
  },
  {
    "id": 714,
    "start_hour": 240,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0714",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_11_78,LEG_11_140,LEG_11_55,LEG_11_185"
  },
  {
    "id": 715,
    "start_hour": 264,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0715",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_12_163,LEG_12_40,LEG_12_46"
  },
  {
    "id": 716,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0716",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_10_37,LEG_10_35"
  },
  {
    "id": 717,
    "start_hour": 265,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0717",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_12_91,LEG_12_88,LEG_12_157,LEG_12_140"
  },
  {
    "id": 718,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0718",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_13_139,LEG_13_28,LEG_13_54"
  },
  {
    "id": 719,
    "start_hour": 218,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0719",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_10_103,LEG_10_166,LEG_10_127"
  },
  {
    "id": 720,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0720",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_11_119,LEG_11_87"
  },
  {
    "id": 721,
    "start_hour": 266,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0721",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_12_82,LEG_12_131"
  },
  {
    "id": 722,
    "start_hour": 288,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0722",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_13_115,LEG_13_123,LEG_13_64"
  },
  {
    "id": 723,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0723",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_10_63,LEG_10_118"
  },
  {
    "id": 724,
    "start_hour": 230,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0724",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_11_147,LEG_11_152,LEG_11_141"
  },
  {
    "id": 725,
    "start_hour": 264,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0725",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_12_115,LEG_12_123,LEG_12_67"
  },
  {
    "id": 726,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0726",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_10_65,LEG_10_66"
  },
  {
    "id": 727,
    "start_hour": 230,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0727",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_11_67,LEG_11_64,LEG_11_37,LEG_11_35"
  },
  {
    "id": 728,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0728",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_10_61,LEG_10_107,LEG_10_11"
  },
  {
    "id": 729,
    "start_hour": 239,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0729",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_11_162,LEG_11_39,LEG_11_62"
  },
  {
    "id": 730,
    "start_hour": 206,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0730",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_10_34,LEG_10_104"
  },
  {
    "id": 731,
    "start_hour": 267,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0731",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_12_133,LEG_12_132"
  },
  {
    "id": 732,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0732",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_12_1,LEG_12_4"
  },
  {
    "id": 733,
    "start_hour": 264,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0733",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_12_153,LEG_12_154"
  },
  {
    "id": 734,
    "start_hour": 265,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0734",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_12_7,LEG_12_63,LEG_12_69,LEG_12_21,LEG_12_96"
  },
  {
    "id": 735,
    "start_hour": 288,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0735",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_13_97,LEG_13_149,LEG_13_69"
  },
  {
    "id": 736,
    "start_hour": 272,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0736",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_12_23,LEG_12_54"
  },
  {
    "id": 737,
    "start_hour": 278,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0737",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_13_21"
  },
  {
    "id": 738,
    "start_hour": 254,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0738",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_12_137,LEG_12_142,LEG_12_3"
  },
  {
    "id": 739,
    "start_hour": 288,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0739",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_13_6"
  },
  {
    "id": 740,
    "start_hour": 264,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0740",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_12_17,LEG_12_33,LEG_12_68"
  },
  {
    "id": 741,
    "start_hour": 287,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0741",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_13_63,LEG_13_34,LEG_13_140"
  },
  {
    "id": 742,
    "start_hour": 252,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0742",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_12_116,LEG_12_59,LEG_12_58,LEG_12_111"
  },
  {
    "id": 743,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0743",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_12_113,LEG_12_84"
  },
  {
    "id": 744,
    "start_hour": 269,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0744",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_12_5,LEG_12_2"
  },
  {
    "id": 745,
    "start_hour": 254,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0745",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_12_110,LEG_12_108,LEG_12_71,LEG_12_127"
  },
  {
    "id": 746,
    "start_hour": 270,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0746",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_12_109,LEG_12_72"
  },
  {
    "id": 747,
    "start_hour": 278,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0747",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_13_110,LEG_13_108,LEG_13_70,LEG_13_127"
  },
  {
    "id": 748,
    "start_hour": 254,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0748",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_12_83,LEG_12_86,LEG_12_80"
  },
  {
    "id": 749,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0749",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_13_83,LEG_13_159,LEG_13_111"
  },
  {
    "id": 750,
    "start_hour": 265,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0750",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_12_70,LEG_12_100,LEG_12_49"
  },
  {
    "id": 751,
    "start_hour": 290,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0751",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_13_41,LEG_13_27,LEG_13_26"
  },
  {
    "id": 752,
    "start_hour": 271,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0752",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_12_19,LEG_12_18"
  },
  {
    "id": 753,
    "start_hour": 278,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0753",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_13_84,LEG_13_87,LEG_13_144"
  },
  {
    "id": 754,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0754",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_14_33,LEG_14_98,LEG_14_167,LEG_14_112"
  },
  {
    "id": 755,
    "start_hour": 336,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0755",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_15_103,LEG_15_159,LEG_15_160"
  },
  {
    "id": 756,
    "start_hour": 274,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0756",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_12_145"
  },
  {
    "id": 757,
    "start_hour": 277,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0757",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_13_75"
  },
  {
    "id": 758,
    "start_hour": 312,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0758",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_14_80,LEG_14_139,LEG_14_56,LEG_14_47"
  },
  {
    "id": 759,
    "start_hour": 336,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0759",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_15_7"
  },
  {
    "id": 760,
    "start_hour": 269,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0760",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_12_135,LEG_12_144,LEG_12_24"
  },
  {
    "id": 761,
    "start_hour": 288,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0761",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_13_77,LEG_13_114"
  },
  {
    "id": 762,
    "start_hour": 312,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0762",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_14_128,LEG_14_120"
  },
  {
    "id": 763,
    "start_hour": 336,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0763",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_15_133"
  },
  {
    "id": 764,
    "start_hour": 272,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0764",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_12_141,LEG_12_136"
  },
  {
    "id": 765,
    "start_hour": 278,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0765",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_13_138,LEG_13_143,LEG_13_141"
  },
  {
    "id": 766,
    "start_hour": 314,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0766",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_14_147,LEG_14_186"
  },
  {
    "id": 767,
    "start_hour": 336,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0767",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_15_195"
  },
  {
    "id": 768,
    "start_hour": 270,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0768",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_12_81,LEG_12_119"
  },
  {
    "id": 769,
    "start_hour": 278,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0769",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_13_128,LEG_13_148,LEG_13_175"
  },
  {
    "id": 770,
    "start_hour": 322,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0770",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_14_189"
  },
  {
    "id": 771,
    "start_hour": 274,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0771",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_12_90"
  },
  {
    "id": 772,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0772",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_13_91,LEG_13_158"
  },
  {
    "id": 773,
    "start_hour": 315,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0773",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_14_162,LEG_14_149"
  },
  {
    "id": 774,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0774",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_15_152,LEG_15_175,LEG_15_122"
  },
  {
    "id": 775,
    "start_hour": 494,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0775",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_22_36,LEG_22_106"
  },
  {
    "id": 776,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0776",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_22_65,LEG_22_122"
  },
  {
    "id": 777,
    "start_hour": 518,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0777",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_23_92,LEG_23_95,LEG_23_4"
  },
  {
    "id": 778,
    "start_hour": 554,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0778",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_24_143,LEG_24_180,LEG_24_40"
  },
  {
    "id": 779,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0779",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_22_67,LEG_22_68"
  },
  {
    "id": 780,
    "start_hour": 518,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0780",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_23_69,LEG_23_66,LEG_23_39,LEG_23_37"
  },
  {
    "id": 781,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0781",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_22_105,LEG_22_59,LEG_22_58"
  },
  {
    "id": 782,
    "start_hour": 531,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0782",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_23_61,LEG_23_76,LEG_23_23,LEG_23_192"
  },
  {
    "id": 783,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0783",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_24_185,LEG_24_110,LEG_24_14"
  },
  {
    "id": 784,
    "start_hour": 564,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0784",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_25_87,LEG_25_96,LEG_25_172,LEG_25_59"
  },
  {
    "id": 785,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0785",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_22_63,LEG_22_111,LEG_22_11"
  },
  {
    "id": 786,
    "start_hour": 527,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0786",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_23_166,LEG_23_176"
  },
  {
    "id": 787,
    "start_hour": 560,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0787",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_24_178"
  },
  {
    "id": 788,
    "start_hour": 566,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0788",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_25_191,LEG_25_186,LEG_25_180,LEG_25_40"
  },
  {
    "id": 789,
    "start_hour": 510,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0789",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_22_76,LEG_22_23,LEG_22_191"
  },
  {
    "id": 790,
    "start_hour": 531,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0790",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_23_168,LEG_23_89"
  },
  {
    "id": 791,
    "start_hour": 554,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0791",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_24_91,LEG_24_145"
  },
  {
    "id": 792,
    "start_hour": 576,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0792",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_25_126,LEG_25_134,LEG_25_74"
  },
  {
    "id": 793,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0793",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_21_163,LEG_21_77,LEG_21_145"
  },
  {
    "id": 794,
    "start_hour": 504,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0794",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_22_126,LEG_22_174"
  },
  {
    "id": 795,
    "start_hour": 528,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0795",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_23_179,LEG_23_144,LEG_23_57,LEG_23_165,LEG_23_82"
  },
  {
    "id": 796,
    "start_hour": 491,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0796",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_21_10"
  },
  {
    "id": 797,
    "start_hour": 508,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0797",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_22_12,LEG_22_53"
  },
  {
    "id": 798,
    "start_hour": 530,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0798",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_23_43,LEG_23_44,LEG_23_52"
  },
  {
    "id": 799,
    "start_hour": 469,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0799",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_21_177,LEG_21_32"
  },
  {
    "id": 800,
    "start_hour": 494,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0800",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_22_121,LEG_22_119,LEG_22_4"
  },
  {
    "id": 801,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0801",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_23_143,LEG_23_187,LEG_23_190"
  },
  {
    "id": 802,
    "start_hour": 489,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0802",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_21_140,LEG_21_142"
  },
  {
    "id": 803,
    "start_hour": 493,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0803",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_22_83"
  },
  {
    "id": 804,
    "start_hour": 528,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0804",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_23_80,LEG_23_81,LEG_23_112"
  },
  {
    "id": 805,
    "start_hour": 483,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0805",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_21_108,LEG_21_132,LEG_21_4"
  },
  {
    "id": 806,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0806",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_22_143,LEG_22_186,LEG_22_189"
  },
  {
    "id": 807,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0807",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_21_45,LEG_21_16,LEG_21_17"
  },
  {
    "id": 808,
    "start_hour": 503,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0808",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_22_20,LEG_22_21,LEG_22_47"
  },
  {
    "id": 809,
    "start_hour": 481,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0809",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_21_183,LEG_21_181,LEG_21_165,LEG_21_82"
  },
  {
    "id": 810,
    "start_hour": 488,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0810",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_21_56,LEG_21_51"
  },
  {
    "id": 811,
    "start_hour": 488,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0811",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_21_113,LEG_21_114"
  },
  {
    "id": 812,
    "start_hour": 469,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0812",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_21_141,LEG_21_42"
  },
  {
    "id": 813,
    "start_hour": 481,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0813",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_21_167,LEG_21_164"
  },
  {
    "id": 814,
    "start_hour": 469,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0814",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_21_54,LEG_21_46"
  },
  {
    "id": 815,
    "start_hour": 482,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0815",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_21_176"
  },
  {
    "id": 816,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0816",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_19_100,LEG_19_110,LEG_19_76,LEG_19_109"
  },
  {
    "id": 817,
    "start_hour": 421,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0817",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_19_49,LEG_19_43"
  },
  {
    "id": 818,
    "start_hour": 421,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0818",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_19_137,LEG_19_37"
  },
  {
    "id": 819,
    "start_hour": 433,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0819",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_19_175,LEG_19_82,LEG_19_39"
  },
  {
    "id": 820,
    "start_hour": 456,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0820",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_20_22,LEG_20_63,LEG_20_174"
  },
  {
    "id": 821,
    "start_hour": 433,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0821",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_19_42,LEG_19_44,LEG_19_174"
  },
  {
    "id": 822,
    "start_hour": 457,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0822",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_20_44,LEG_20_45,LEG_20_50"
  },
  {
    "id": 823,
    "start_hour": 434,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0823",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_19_168"
  },
  {
    "id": 824,
    "start_hour": 464,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0824",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_20_166"
  },
  {
    "id": 825,
    "start_hour": 443,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0825",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_19_10"
  },
  {
    "id": 826,
    "start_hour": 460,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0826",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_20_11"
  },
  {
    "id": 827,
    "start_hour": 470,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0827",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_21_191,LEG_21_186,LEG_21_50,LEG_21_55"
  },
  {
    "id": 828,
    "start_hour": 442,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0828",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_19_38"
  },
  {
    "id": 829,
    "start_hour": 444,
    "duration_hours": 27,
    "required_skill": "A320",
    "gerad_duty_id": "D0829",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_20_163,LEG_20_61,LEG_20_77,LEG_20_104,LEG_20_105"
  },
  {
    "id": 830,
    "start_hour": 436,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0830",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_19_111,LEG_19_29,LEG_19_3"
  },
  {
    "id": 831,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0831",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_20_134,LEG_20_72,LEG_20_103,LEG_20_154,LEG_20_73"
  },
  {
    "id": 832,
    "start_hour": 302,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0832",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_14_35,LEG_14_104"
  },
  {
    "id": 833,
    "start_hour": 311,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0833",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_14_22,LEG_14_166,LEG_14_58"
  },
  {
    "id": 834,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0834",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_14_66,LEG_14_67"
  },
  {
    "id": 835,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0835",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_14_103,LEG_14_88,LEG_14_89"
  },
  {
    "id": 836,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0836",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_15_91,LEG_15_28,LEG_15_60"
  },
  {
    "id": 837,
    "start_hour": 318,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0837",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_14_76,LEG_14_21,LEG_14_185"
  },
  {
    "id": 838,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0838",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_15_168,LEG_15_131"
  },
  {
    "id": 839,
    "start_hour": 362,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0839",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_16_123,LEG_16_145"
  },
  {
    "id": 840,
    "start_hour": 384,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0840",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_17_126,LEG_17_134,LEG_17_74"
  },
  {
    "id": 841,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0841",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_14_34,LEG_14_148,LEG_14_25"
  },
  {
    "id": 842,
    "start_hour": 336,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0842",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_15_171,LEG_15_21,LEG_15_47,LEG_15_182"
  },
  {
    "id": 843,
    "start_hour": 361,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0843",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_16_49,LEG_16_81,LEG_16_112,LEG_16_189"
  },
  {
    "id": 844,
    "start_hour": 384,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0844",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_17_179,LEG_17_180,LEG_17_40"
  },
  {
    "id": 845,
    "start_hour": 303,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0845",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_14_69,LEG_14_157"
  },
  {
    "id": 846,
    "start_hour": 325,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0846",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_15_177,LEG_15_32"
  },
  {
    "id": 847,
    "start_hour": 350,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0847",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_16_121,LEG_16_119,LEG_16_28,LEG_16_60"
  },
  {
    "id": 848,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0848",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_14_64,LEG_14_117"
  },
  {
    "id": 849,
    "start_hour": 326,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0849",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_15_139,LEG_15_161,LEG_15_194"
  },
  {
    "id": 850,
    "start_hour": 370,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0850",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_16_196"
  },
  {
    "id": 851,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0851",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_17_121,LEG_17_119,LEG_17_28,LEG_17_60"
  },
  {
    "id": 852,
    "start_hour": 5,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0852",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_01_41,LEG_01_45"
  },
  {
    "id": 853,
    "start_hour": 8,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0853",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_01_153"
  },
  {
    "id": 854,
    "start_hour": 1,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0854",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_01_143,LEG_01_140,LEG_01_157"
  },
  {
    "id": 855,
    "start_hour": 25,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0855",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_02_47,LEG_02_48,LEG_02_53"
  },
  {
    "id": 856,
    "start_hour": 6,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0856",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_01_159,LEG_01_7,LEG_01_44"
  },
  {
    "id": 857,
    "start_hour": 26,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0857",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_02_41,LEG_02_42,LEG_02_50"
  },
  {
    "id": 858,
    "start_hour": 3,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0858",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_01_90,LEG_01_109,LEG_01_3"
  },
  {
    "id": 859,
    "start_hour": 26,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0859",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_02_138,LEG_02_182,LEG_02_185"
  },
  {
    "id": 860,
    "start_hour": 6,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0860",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_01_35,LEG_01_43,LEG_01_166"
  },
  {
    "id": 861,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0861",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_02_180,LEG_02_179,LEG_02_9"
  },
  {
    "id": 862,
    "start_hour": 4,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0862",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_01_162,LEG_01_49,LEG_01_63"
  },
  {
    "id": 863,
    "start_hour": 23,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0863",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_02_71,LEG_02_70,LEG_02_183"
  },
  {
    "id": 864,
    "start_hour": 2,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0864",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_01_152"
  },
  {
    "id": 865,
    "start_hour": 32,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0865",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_02_173"
  },
  {
    "id": 866,
    "start_hour": 1,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0866",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_01_36,LEG_01_38,LEG_01_116,LEG_01_117"
  },
  {
    "id": 867,
    "start_hour": 8,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0867",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_01_46,LEG_01_42"
  },
  {
    "id": 868,
    "start_hour": 1,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0868",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_01_158,LEG_01_156,LEG_01_141,LEG_01_70"
  },
  {
    "id": 869,
    "start_hour": 4,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0869",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_01_69,LEG_01_92,LEG_01_93,LEG_01_94"
  },
  {
    "id": 870,
    "start_hour": 11,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0870",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_01_8"
  },
  {
    "id": 871,
    "start_hour": 28,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0871",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_02_12,LEG_02_51"
  },
  {
    "id": 872,
    "start_hour": 50,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0872",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_03_41,LEG_03_180,LEG_03_9"
  },
  {
    "id": 873,
    "start_hour": 4,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0873",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_01_155,LEG_01_33,LEG_01_48"
  },
  {
    "id": 874,
    "start_hour": 27,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0874",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_02_59,LEG_02_74,LEG_02_22"
  },
  {
    "id": 875,
    "start_hour": 38,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D0875",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_03_187,LEG_03_182,LEG_03_48,LEG_03_53"
  },
  {
    "id": 876,
    "start_hour": 4,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0876",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_01_119,LEG_01_47,LEG_01_164"
  },
  {
    "id": 877,
    "start_hour": 24,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0877",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_02_174,LEG_02_139,LEG_02_55,LEG_02_160,LEG_02_79"
  },
  {
    "id": 878,
    "start_hour": 3,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0878",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_01_138,LEG_01_65,LEG_01_150"
  },
  {
    "id": 879,
    "start_hour": 27,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0879",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_02_163,LEG_02_18,LEG_02_17"
  },
  {
    "id": 880,
    "start_hour": 48,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0880",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_03_165,LEG_03_64,LEG_03_73"
  },
  {
    "id": 881,
    "start_hour": 71,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0881",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_04_71,LEG_04_61,LEG_04_107"
  },
  {
    "id": 882,
    "start_hour": 630,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0882",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_27_86,LEG_27_85"
  },
  {
    "id": 883,
    "start_hour": 637,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0883",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_28_39,LEG_28_78,LEG_28_126"
  },
  {
    "id": 884,
    "start_hour": 673,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0884",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_29_127,LEG_29_130"
  },
  {
    "id": 885,
    "start_hour": 697,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0885",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_30_126,LEG_30_115,LEG_30_117"
  },
  {
    "id": 886,
    "start_hour": 635,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0886",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_27_70"
  },
  {
    "id": 887,
    "start_hour": 639,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0887",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_28_85,LEG_28_88,LEG_28_121"
  },
  {
    "id": 888,
    "start_hour": 675,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0888",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_29_123"
  },
  {
    "id": 889,
    "start_hour": 685,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0889",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_30_118,LEG_30_154,LEG_30_168,LEG_30_82"
  },
  {
    "id": 890,
    "start_hour": 634,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0890",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_27_95"
  },
  {
    "id": 891,
    "start_hour": 637,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D0891",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_28_113,LEG_28_192,LEG_28_81"
  },
  {
    "id": 892,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0892",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_29_85,LEG_29_35"
  },
  {
    "id": 893,
    "start_hour": 696,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0893",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_30_161,LEG_30_63,LEG_30_32"
  },
  {
    "id": 894,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0894",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_27_76,LEG_27_75"
  },
  {
    "id": 895,
    "start_hour": 625,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0895",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_27_136,LEG_27_156,LEG_27_121"
  },
  {
    "id": 896,
    "start_hour": 649,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0896",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_28_41,LEG_28_145,LEG_28_12,LEG_28_184"
  },
  {
    "id": 897,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0897",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_29_195,LEG_29_62,LEG_29_6"
  },
  {
    "id": 898,
    "start_hour": 696,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0898",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_30_15,LEG_30_29"
  },
  {
    "id": 899,
    "start_hour": 633,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0899",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_27_6"
  },
  {
    "id": 900,
    "start_hour": 636,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0900",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_28_202,LEG_28_196,LEG_28_185,LEG_28_95"
  },
  {
    "id": 901,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0901",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_29_168,LEG_29_52"
  },
  {
    "id": 902,
    "start_hour": 696,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0902",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_30_133"
  },
  {
    "id": 903,
    "start_hour": 628,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0903",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_27_124,LEG_27_123"
  },
  {
    "id": 904,
    "start_hour": 626,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0904",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_27_128,LEG_27_0"
  },
  {
    "id": 905,
    "start_hour": 628,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0905",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_27_129,LEG_27_131"
  },
  {
    "id": 906,
    "start_hour": 612,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0906",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_27_5"
  },
  {
    "id": 907,
    "start_hour": 637,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0907",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_28_47"
  },
  {
    "id": 908,
    "start_hour": 634,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0908",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_27_26"
  },
  {
    "id": 909,
    "start_hour": 637,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0909",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_28_74,LEG_28_23"
  },
  {
    "id": 910,
    "start_hour": 614,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0910",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_27_94,LEG_27_97,LEG_27_20"
  },
  {
    "id": 911,
    "start_hour": 649,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0911",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_28_144,LEG_28_97,LEG_28_169"
  },
  {
    "id": 912,
    "start_hour": 630,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0912",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_27_63,LEG_27_65"
  },
  {
    "id": 913,
    "start_hour": 637,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0913",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_28_64,LEG_28_62,LEG_28_63,LEG_28_32"
  },
  {
    "id": 914,
    "start_hour": 614,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0914",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_27_88,LEG_27_89,LEG_27_135,LEG_27_96"
  },
  {
    "id": 915,
    "start_hour": 630,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0915",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_27_2,LEG_27_3"
  },
  {
    "id": 916,
    "start_hour": 630,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0916",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_27_34,LEG_27_37"
  },
  {
    "id": 917,
    "start_hour": 637,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0917",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_28_159"
  },
  {
    "id": 918,
    "start_hour": 661,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0918",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_29_48"
  },
  {
    "id": 919,
    "start_hour": 614,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0919",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_27_110,LEG_27_112"
  },
  {
    "id": 920,
    "start_hour": 637,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0920",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_28_134,LEG_28_136,LEG_28_93"
  },
  {
    "id": 921,
    "start_hour": 673,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0921",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_29_129"
  },
  {
    "id": 922,
    "start_hour": 636,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0922",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_27_74"
  },
  {
    "id": 923,
    "start_hour": 651,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D0923",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_28_94"
  },
  {
    "id": 924,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D0924",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_29_65,LEG_29_63,LEG_29_78,LEG_29_81"
  },
  {
    "id": 925,
    "start_hour": 629,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0925",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_27_23,LEG_27_54"
  },
  {
    "id": 926,
    "start_hour": 637,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0926",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_28_108,LEG_28_156,LEG_28_141,LEG_28_205"
  },
  {
    "id": 927,
    "start_hour": 634,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0927",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_27_114"
  },
  {
    "id": 928,
    "start_hour": 637,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0928",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_28_90,LEG_28_86"
  },
  {
    "id": 929,
    "start_hour": 663,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0929",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_29_121,LEG_29_120"
  },
  {
    "id": 930,
    "start_hour": 628,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0930",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_27_133,LEG_27_67"
  },
  {
    "id": 931,
    "start_hour": 637,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0931",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_28_131,LEG_28_111,LEG_28_137"
  },
  {
    "id": 932,
    "start_hour": 675,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0932",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_29_145"
  },
  {
    "id": 933,
    "start_hour": 685,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0933",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_30_135,LEG_30_113,LEG_30_112,LEG_30_111"
  },
  {
    "id": 934,
    "start_hour": 632,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0934",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_27_36,LEG_27_125"
  },
  {
    "id": 935,
    "start_hour": 639,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0935",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_28_138,LEG_28_135"
  },
  {
    "id": 936,
    "start_hour": 661,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0936",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_29_40,LEG_29_79,LEG_29_95"
  },
  {
    "id": 937,
    "start_hour": 697,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0937",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_30_128"
  },
  {
    "id": 938,
    "start_hour": 721,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0938",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_31_172,LEG_31_83"
  },
  {
    "id": 939,
    "start_hour": 712,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0939",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_31_2,LEG_31_23"
  },
  {
    "id": 940,
    "start_hour": 727,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0940",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_31_146,LEG_31_208"
  },
  {
    "id": 941,
    "start_hour": 728,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0941",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_31_171,LEG_31_122"
  },
  {
    "id": 942,
    "start_hour": 724,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0942",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_31_161,LEG_31_164"
  },
  {
    "id": 943,
    "start_hour": 723,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0943",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_31_94,LEG_31_92"
  },
  {
    "id": 944,
    "start_hour": 724,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0944",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_31_156,LEG_31_154"
  },
  {
    "id": 945,
    "start_hour": 726,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0945",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_31_1,LEG_31_3"
  },
  {
    "id": 946,
    "start_hour": 711,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D0946",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_31_133,LEG_31_137"
  },
  {
    "id": 947,
    "start_hour": 314,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0947",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_14_170"
  },
  {
    "id": 948,
    "start_hour": 313,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0948",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_14_161,LEG_14_159"
  },
  {
    "id": 949,
    "start_hour": 301,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0949",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_14_53,LEG_14_45"
  },
  {
    "id": 950,
    "start_hour": 301,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0950",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_14_136,LEG_14_41"
  },
  {
    "id": 951,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0951",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_14_44,LEG_14_15,LEG_14_168"
  },
  {
    "id": 952,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0952",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_15_185,LEG_15_184,LEG_15_9"
  },
  {
    "id": 953,
    "start_hour": 313,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0953",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_14_40,LEG_14_63,LEG_14_74"
  },
  {
    "id": 954,
    "start_hour": 335,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0954",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_15_73,LEG_15_72,LEG_15_188"
  },
  {
    "id": 955,
    "start_hour": 320,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0955",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_14_55,LEG_14_50"
  },
  {
    "id": 956,
    "start_hour": 313,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0956",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_14_177,LEG_14_175,LEG_14_160,LEG_14_82"
  },
  {
    "id": 957,
    "start_hour": 320,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0957",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_14_109,LEG_14_110"
  },
  {
    "id": 958,
    "start_hour": 323,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0958",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_14_10"
  },
  {
    "id": 959,
    "start_hour": 340,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0959",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_15_12,LEG_15_53"
  },
  {
    "id": 960,
    "start_hour": 362,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0960",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_16_43,LEG_16_44,LEG_16_52"
  },
  {
    "id": 961,
    "start_hour": 301,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0961",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_14_171,LEG_14_30"
  },
  {
    "id": 962,
    "start_hour": 326,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D0962",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_15_92,LEG_15_95,LEG_15_4"
  },
  {
    "id": 963,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0963",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_16_143,LEG_16_187,LEG_16_190"
  },
  {
    "id": 964,
    "start_hour": 323,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0964",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_14_52"
  },
  {
    "id": 965,
    "start_hour": 338,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0965",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_15_43,LEG_15_140,LEG_15_142"
  },
  {
    "id": 966,
    "start_hour": 315,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0966",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_14_106,LEG_14_127"
  },
  {
    "id": 967,
    "start_hour": 324,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0967",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_15_87,LEG_15_96,LEG_15_147,LEG_15_146"
  },
  {
    "id": 968,
    "start_hour": 360,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0968",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_16_169,LEG_16_59,LEG_16_75"
  },
  {
    "id": 969,
    "start_hour": 383,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0969",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_17_73,LEG_17_72,LEG_17_188"
  },
  {
    "id": 970,
    "start_hour": 321,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0970",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_14_135,LEG_14_137"
  },
  {
    "id": 971,
    "start_hour": 325,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0971",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_15_83"
  },
  {
    "id": 972,
    "start_hour": 360,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0972",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_16_80,LEG_16_144,LEG_16_57,LEG_16_182"
  },
  {
    "id": 973,
    "start_hour": 385,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0973",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_17_49,LEG_17_187,LEG_17_190"
  },
  {
    "id": 974,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0974",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_09_63,LEG_09_118"
  },
  {
    "id": 975,
    "start_hour": 206,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0975",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_10_147,LEG_10_152,LEG_10_87"
  },
  {
    "id": 976,
    "start_hour": 242,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0976",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_11_89"
  },
  {
    "id": 977,
    "start_hour": 252,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D0977",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_12_78,LEG_12_87,LEG_12_156,LEG_12_53"
  },
  {
    "id": 978,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0978",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_09_65,LEG_09_66"
  },
  {
    "id": 979,
    "start_hour": 206,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0979",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_10_67,LEG_10_177,LEG_10_161,LEG_10_80"
  },
  {
    "id": 980,
    "start_hour": 229,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0980",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_11_81"
  },
  {
    "id": 981,
    "start_hour": 264,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0981",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_12_73,LEG_12_164,LEG_12_35"
  },
  {
    "id": 982,
    "start_hour": 194,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0982",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_09_103,LEG_09_177,LEG_09_46"
  },
  {
    "id": 983,
    "start_hour": 216,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0983",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_10_7,LEG_10_171"
  },
  {
    "id": 984,
    "start_hour": 240,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0984",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_11_175,LEG_11_176,LEG_11_38"
  },
  {
    "id": 985,
    "start_hour": 194,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0985",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_09_33,LEG_09_149"
  },
  {
    "id": 986,
    "start_hour": 204,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0986",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_10_13,LEG_10_0,LEG_10_130,LEG_10_72"
  },
  {
    "id": 987,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0987",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_09_61,LEG_09_107,LEG_09_11"
  },
  {
    "id": 988,
    "start_hour": 215,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0988",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_10_162,LEG_10_39,LEG_10_62"
  },
  {
    "id": 989,
    "start_hour": 198,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0989",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_09_37,LEG_09_35"
  },
  {
    "id": 990,
    "start_hour": 182,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0990",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_09_34,LEG_09_104"
  },
  {
    "id": 991,
    "start_hour": 182,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0991",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_09_67,LEG_09_60"
  },
  {
    "id": 992,
    "start_hour": 288,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0992",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_13_154,LEG_13_155"
  },
  {
    "id": 993,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0993",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_13_1,LEG_13_4"
  },
  {
    "id": 994,
    "start_hour": 280,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0994",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_13_0,LEG_13_88"
  },
  {
    "id": 995,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0995",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_13_89,LEG_13_93"
  },
  {
    "id": 996,
    "start_hour": 291,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0996",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_13_31,LEG_13_30,LEG_13_25"
  },
  {
    "id": 997,
    "start_hour": 312,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0997",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_14_165,LEG_14_29,LEG_14_28"
  },
  {
    "id": 998,
    "start_hour": 291,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0998",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_13_18,LEG_13_14,LEG_13_15"
  },
  {
    "id": 999,
    "start_hour": 313,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0999",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_14_143,LEG_14_113,LEG_14_111"
  },
  {
    "id": 1000,
    "start_hour": 292,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1000",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_13_125,LEG_13_24"
  },
  {
    "id": 1001,
    "start_hour": 301,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D1001",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_14_8,LEG_14_105,LEG_14_158,LEG_14_77"
  },
  {
    "id": 1002,
    "start_hour": 295,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1002",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_13_17,LEG_13_16"
  },
  {
    "id": 1003,
    "start_hour": 302,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1003",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_14_124,LEG_14_123"
  },
  {
    "id": 1004,
    "start_hour": 297,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1004",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_13_86,LEG_13_61"
  },
  {
    "id": 1005,
    "start_hour": 302,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1005",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_14_23"
  },
  {
    "id": 1006,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1006",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_13_82,LEG_13_119"
  },
  {
    "id": 1007,
    "start_hour": 302,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1007",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_14_134,LEG_14_156"
  },
  {
    "id": 1008,
    "start_hour": 293,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1008",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_13_5,LEG_13_2"
  },
  {
    "id": 1009,
    "start_hour": 291,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1009",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_13_135,LEG_13_134"
  },
  {
    "id": 1010,
    "start_hour": 300,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1010",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_14_87,LEG_14_96,LEG_14_19,LEG_14_46,LEG_14_11"
  },
  {
    "id": 1011,
    "start_hour": 335,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1011",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_15_166,LEG_15_45,LEG_15_16"
  },
  {
    "id": 1012,
    "start_hour": 298,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1012",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_13_92"
  },
  {
    "id": 1013,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1013",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_14_99,LEG_14_16"
  },
  {
    "id": 1014,
    "start_hour": 335,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1014",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_15_20,LEG_15_104,LEG_15_102"
  },
  {
    "id": 1015,
    "start_hour": 293,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1015",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_13_136,LEG_13_145,LEG_13_29"
  },
  {
    "id": 1016,
    "start_hour": 312,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1016",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_14_84,LEG_14_130"
  },
  {
    "id": 1017,
    "start_hour": 337,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1017",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_15_137,LEG_15_136,LEG_15_26"
  },
  {
    "id": 1018,
    "start_hour": 296,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1018",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_13_142,LEG_13_137"
  },
  {
    "id": 1019,
    "start_hour": 302,
    "duration_hours": 18,
    "required_skill": "A320",
    "gerad_duty_id": "D1019",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_14_146,LEG_14_151"
  },
  {
    "id": 1020,
    "start_hour": 325,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1020",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_15_8,LEG_15_107,LEG_15_163,LEG_15_77"
  },
  {
    "id": 1021,
    "start_hour": 294,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1021",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_13_109,LEG_13_71"
  },
  {
    "id": 1022,
    "start_hour": 302,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1022",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_14_116,LEG_14_114,LEG_14_78,LEG_14_133"
  },
  {
    "id": 1023,
    "start_hour": 293,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1023",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_13_107,LEG_13_104"
  },
  {
    "id": 1024,
    "start_hour": 300,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1024",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_14_122,LEG_14_179,LEG_14_183"
  },
  {
    "id": 1025,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1025",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_15_117,LEG_15_98,LEG_15_173,LEG_15_17"
  },
  {
    "id": 1026,
    "start_hour": 359,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1026",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_16_20,LEG_16_104,LEG_16_102"
  },
  {
    "id": 1027,
    "start_hour": 291,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1027",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_13_156,LEG_13_53,LEG_13_52"
  },
  {
    "id": 1028,
    "start_hour": 315,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1028",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_14_60,LEG_14_57"
  },
  {
    "id": 1029,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1029",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_15_109,LEG_15_110,LEG_15_14"
  },
  {
    "id": 1030,
    "start_hour": 348,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1030",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_16_87,LEG_16_96,LEG_16_172,LEG_16_88"
  },
  {
    "id": 1031,
    "start_hour": 601,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1031",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_26_8,LEG_26_67,LEG_26_72"
  },
  {
    "id": 1032,
    "start_hour": 628,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1032",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_27_159,LEG_27_58"
  },
  {
    "id": 1033,
    "start_hour": 652,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1033",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_28_66,LEG_28_67,LEG_28_49"
  },
  {
    "id": 1034,
    "start_hour": 660,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D1034",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_29_50,LEG_29_115,LEG_29_124,LEG_29_197"
  },
  {
    "id": 1035,
    "start_hour": 612,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1035",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_26_3"
  },
  {
    "id": 1036,
    "start_hour": 627,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1036",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_27_50,LEG_27_11,LEG_27_46,LEG_27_8"
  },
  {
    "id": 1037,
    "start_hour": 647,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1037",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_28_128,LEG_28_187,LEG_28_82"
  },
  {
    "id": 1038,
    "start_hour": 605,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1038",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_26_79,LEG_26_80"
  },
  {
    "id": 1039,
    "start_hour": 600,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1039",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_26_156,LEG_26_157"
  },
  {
    "id": 1040,
    "start_hour": 605,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1040",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_26_140,LEG_26_148"
  },
  {
    "id": 1041,
    "start_hour": 602,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1041",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_26_1,LEG_26_4"
  },
  {
    "id": 1042,
    "start_hour": 612,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1042",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_26_179"
  },
  {
    "id": 1043,
    "start_hour": 625,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1043",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_27_103,LEG_27_91,LEG_27_93"
  },
  {
    "id": 1044,
    "start_hour": 605,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1044",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_26_5,LEG_26_2"
  },
  {
    "id": 1045,
    "start_hour": 607,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1045",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_26_19,LEG_26_18"
  },
  {
    "id": 1046,
    "start_hour": 610,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1046",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_26_125"
  },
  {
    "id": 1047,
    "start_hour": 625,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1047",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_27_101,LEG_27_104"
  },
  {
    "id": 1048,
    "start_hour": 649,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1048",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_28_123,LEG_28_112,LEG_28_114"
  },
  {
    "id": 1049,
    "start_hour": 611,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1049",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_26_7"
  },
  {
    "id": 1050,
    "start_hour": 629,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1050",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_27_18,LEG_27_17"
  },
  {
    "id": 1051,
    "start_hour": 608,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1051",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_26_145,LEG_26_141"
  },
  {
    "id": 1052,
    "start_hour": 614,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1052",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_27_71,LEG_27_73"
  },
  {
    "id": 1053,
    "start_hour": 637,
    "duration_hours": 17,
    "required_skill": "A319",
    "gerad_duty_id": "D1053",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_28_148,LEG_28_0"
  },
  {
    "id": 1054,
    "start_hour": 607,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1054",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_26_162,LEG_26_116"
  },
  {
    "id": 1055,
    "start_hour": 614,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1055",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_27_21,LEG_27_35,LEG_27_19"
  },
  {
    "id": 1056,
    "start_hour": 638,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1056",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_28_4,LEG_28_13,LEG_28_102,LEG_28_171"
  },
  {
    "id": 1057,
    "start_hour": 601,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1057",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_26_74,LEG_26_103,LEG_26_54"
  },
  {
    "id": 1058,
    "start_hour": 625,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1058",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_27_119,LEG_27_120,LEG_27_12,LEG_27_147"
  },
  {
    "id": 1059,
    "start_hour": 651,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1059",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_28_191,LEG_28_16,LEG_28_55"
  },
  {
    "id": 1060,
    "start_hour": 673,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1060",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_29_149,LEG_29_99,LEG_29_173"
  },
  {
    "id": 1061,
    "start_hour": 610,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1061",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_26_93"
  },
  {
    "id": 1062,
    "start_hour": 627,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1062",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_27_78,LEG_27_66"
  },
  {
    "id": 1063,
    "start_hour": 651,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1063",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_28_84"
  },
  {
    "id": 1064,
    "start_hour": 661,
    "duration_hours": 17,
    "required_skill": "A321",
    "gerad_duty_id": "D1064",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_29_153,LEG_29_0"
  },
  {
    "id": 1065,
    "start_hour": 612,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1065",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_26_21"
  },
  {
    "id": 1066,
    "start_hour": 629,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1066",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_27_41,LEG_27_56,LEG_27_43"
  },
  {
    "id": 1067,
    "start_hour": 636,
    "duration_hours": 19,
    "required_skill": "A319",
    "gerad_duty_id": "D1067",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_28_180,LEG_28_166"
  },
  {
    "id": 1068,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1068",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_29_136,LEG_29_114,LEG_29_113,LEG_29_112"
  },
  {
    "id": 1069,
    "start_hour": 610,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1069",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_26_182"
  },
  {
    "id": 1070,
    "start_hour": 630,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1070",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_27_32,LEG_27_99"
  },
  {
    "id": 1071,
    "start_hour": 651,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1071",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_28_119,LEG_28_34"
  },
  {
    "id": 1072,
    "start_hour": 672,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1072",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_29_162,LEG_29_64,LEG_29_33"
  },
  {
    "id": 1073,
    "start_hour": 651,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1073",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_28_69,LEG_28_27,LEG_28_31"
  },
  {
    "id": 1074,
    "start_hour": 678,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1074",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_29_34"
  },
  {
    "id": 1075,
    "start_hour": 685,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1075",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_30_39,LEG_30_59,LEG_30_193"
  },
  {
    "id": 1076,
    "start_hour": 651,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1076",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_28_42,LEG_28_25"
  },
  {
    "id": 1077,
    "start_hour": 661,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1077",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_29_119,LEG_29_155,LEG_29_169,LEG_29_207,LEG_29_206"
  },
  {
    "id": 1078,
    "start_hour": 700,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1078",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_30_201,LEG_30_72"
  },
  {
    "id": 1079,
    "start_hour": 724,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1079",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_31_67,LEG_31_47,LEG_31_45"
  },
  {
    "id": 1080,
    "start_hour": 639,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1080",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_28_43,LEG_28_20"
  },
  {
    "id": 1081,
    "start_hour": 663,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1081",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_29_133,LEG_29_137"
  },
  {
    "id": 1082,
    "start_hour": 685,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1082",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_30_110,LEG_30_17,LEG_30_105"
  },
  {
    "id": 1083,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1083",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_28_199,LEG_28_139"
  },
  {
    "id": 1084,
    "start_hour": 663,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1084",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_29_39,LEG_29_37,LEG_29_88,LEG_29_77"
  },
  {
    "id": 1085,
    "start_hour": 686,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1085",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_30_51"
  },
  {
    "id": 1086,
    "start_hour": 720,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1086",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_31_134,LEG_31_128,LEG_31_74"
  },
  {
    "id": 1087,
    "start_hour": 648,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1087",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_28_22,LEG_28_130"
  },
  {
    "id": 1088,
    "start_hour": 675,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1088",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_29_138,LEG_29_27"
  },
  {
    "id": 1089,
    "start_hour": 684,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1089",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_30_71"
  },
  {
    "id": 1090,
    "start_hour": 639,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1090",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_28_70,LEG_28_68"
  },
  {
    "id": 1091,
    "start_hour": 639,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1091",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_28_142,LEG_28_201"
  },
  {
    "id": 1092,
    "start_hour": 651,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1092",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_28_200,LEG_28_105,LEG_28_177"
  },
  {
    "id": 1093,
    "start_hour": 661,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1093",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_29_148"
  },
  {
    "id": 1094,
    "start_hour": 640,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1094",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_28_161,LEG_28_155"
  },
  {
    "id": 1095,
    "start_hour": 662,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1095",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_29_167,LEG_29_100"
  },
  {
    "id": 1096,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1096",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_28_195,LEG_28_197"
  },
  {
    "id": 1097,
    "start_hour": 655,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1097",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_28_46,LEG_28_44"
  },
  {
    "id": 1098,
    "start_hour": 697,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1098",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_30_18,LEG_30_45,LEG_30_67,LEG_30_50"
  },
  {
    "id": 1099,
    "start_hour": 720,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1099",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_31_15,LEG_31_30"
  },
  {
    "id": 1100,
    "start_hour": 698,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1100",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_30_104,LEG_30_179"
  },
  {
    "id": 1101,
    "start_hour": 720,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1101",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_31_21,LEG_31_78,LEG_31_81"
  },
  {
    "id": 1102,
    "start_hour": 687,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1102",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_30_132,LEG_30_136"
  },
  {
    "id": 1103,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1103",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_31_91,LEG_31_87"
  },
  {
    "id": 1104,
    "start_hour": 702,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1104",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_30_77,LEG_30_80"
  },
  {
    "id": 1105,
    "start_hour": 709,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1105",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_31_65,LEG_31_63,LEG_31_25,LEG_31_66"
  },
  {
    "id": 1106,
    "start_hour": 697,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1106",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_30_171,LEG_30_172"
  },
  {
    "id": 1107,
    "start_hour": 704,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1107",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_30_83,LEG_30_143"
  },
  {
    "id": 1108,
    "start_hour": 711,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1108",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_31_39,LEG_31_37,LEG_31_84,LEG_31_144"
  },
  {
    "id": 1109,
    "start_hour": 704,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1109",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_30_21,LEG_30_19"
  },
  {
    "id": 1110,
    "start_hour": 711,
    "duration_hours": 18,
    "required_skill": "A319",
    "gerad_duty_id": "D1110",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_31_86,LEG_31_89"
  },
  {
    "id": 1111,
    "start_hour": 703,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1111",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_30_145,LEG_30_207"
  },
  {
    "id": 1112,
    "start_hour": 685,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1112",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_30_90,LEG_30_86"
  },
  {
    "id": 1113,
    "start_hour": 711,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1113",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_31_121,LEG_31_120"
  },
  {
    "id": 1114,
    "start_hour": 706,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1114",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_30_28"
  },
  {
    "id": 1115,
    "start_hour": 709,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1115",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_31_75,LEG_31_24"
  },
  {
    "id": 1116,
    "start_hour": 704,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1116",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_30_170,LEG_30_121"
  },
  {
    "id": 1117,
    "start_hour": 688,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1117",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_30_2,LEG_30_22"
  },
  {
    "id": 1118,
    "start_hour": 708,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1118",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_30_89"
  },
  {
    "id": 1119,
    "start_hour": 723,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1119",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_31_96"
  },
  {
    "id": 1120,
    "start_hour": 700,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1120",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_30_155,LEG_30_153"
  },
  {
    "id": 1121,
    "start_hour": 702,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1121",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_30_1,LEG_30_3"
  },
  {
    "id": 1122,
    "start_hour": 700,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1122",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_30_160,LEG_30_163"
  },
  {
    "id": 1123,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1123",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_30_93,LEG_30_91"
  },
  {
    "id": 1124,
    "start_hour": 685,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1124",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_30_138,LEG_30_197,LEG_30_198,LEG_30_200"
  },
  {
    "id": 1125,
    "start_hour": 723,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1125",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_31_99,LEG_31_173"
  },
  {
    "id": 1126,
    "start_hour": 685,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1126",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_30_152,LEG_30_0,LEG_30_141"
  },
  {
    "id": 1127,
    "start_hour": 723,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1127",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_31_145,LEG_31_38,LEG_31_157"
  },
  {
    "id": 1128,
    "start_hour": 685,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1128",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_30_116,LEG_30_195,LEG_30_94"
  },
  {
    "id": 1129,
    "start_hour": 721,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1129",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_31_129,LEG_31_113,LEG_31_112"
  },
  {
    "id": 1130,
    "start_hour": 702,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1130",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_30_35,LEG_30_40"
  },
  {
    "id": 1131,
    "start_hour": 709,
    "duration_hours": 17,
    "required_skill": "A321",
    "gerad_duty_id": "D1131",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_31_40,LEG_31_79"
  },
  {
    "id": 1132,
    "start_hour": 701,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1132",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_30_24,LEG_30_65"
  },
  {
    "id": 1133,
    "start_hour": 710,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1133",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_31_4,LEG_31_13,LEG_31_104,LEG_31_175"
  },
  {
    "id": 1134,
    "start_hour": 704,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1134",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_30_37,LEG_30_156"
  },
  {
    "id": 1135,
    "start_hour": 711,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1135",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_31_143,LEG_31_140"
  },
  {
    "id": 1136,
    "start_hour": 663,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1136",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_29_44,LEG_29_21"
  },
  {
    "id": 1137,
    "start_hour": 687,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D1137",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_30_85,LEG_30_88"
  },
  {
    "id": 1138,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1138",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_31_139,LEG_31_198,LEG_31_199,LEG_31_201"
  },
  {
    "id": 1139,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1139",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_29_203,LEG_29_144"
  },
  {
    "id": 1140,
    "start_hour": 687,
    "duration_hours": 16,
    "required_skill": "A321",
    "gerad_duty_id": "D1140",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_30_38,LEG_30_36"
  },
  {
    "id": 1141,
    "start_hour": 709,
    "duration_hours": 17,
    "required_skill": "A321",
    "gerad_duty_id": "D1141",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_31_153,LEG_31_0"
  },
  {
    "id": 1142,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1142",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_29_70,LEG_29_28,LEG_29_82"
  },
  {
    "id": 1143,
    "start_hour": 699,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1143",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_30_84,LEG_30_26"
  },
  {
    "id": 1144,
    "start_hour": 708,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1144",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_31_72"
  },
  {
    "id": 1145,
    "start_hour": 675,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1145",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_29_43,LEG_29_26"
  },
  {
    "id": 1146,
    "start_hour": 685,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1146",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_30_64,LEG_30_62,LEG_30_127,LEG_30_73"
  },
  {
    "id": 1147,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1147",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_29_204,LEG_29_107,LEG_29_55"
  },
  {
    "id": 1148,
    "start_hour": 696,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1148",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_30_102,LEG_30_14,LEG_30_151"
  },
  {
    "id": 1149,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1149",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_29_199,LEG_29_201"
  },
  {
    "id": 1150,
    "start_hour": 663,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1150",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_29_147,LEG_29_205"
  },
  {
    "id": 1151,
    "start_hour": 663,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1151",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_29_71,LEG_29_69"
  },
  {
    "id": 1152,
    "start_hour": 684,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1152",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_29_90"
  },
  {
    "id": 1153,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1153",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_30_95,LEG_30_34"
  },
  {
    "id": 1154,
    "start_hour": 720,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1154",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_31_162,LEG_31_64,LEG_31_33"
  },
  {
    "id": 1155,
    "start_hour": 683,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1155",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_29_32"
  },
  {
    "id": 1156,
    "start_hour": 702,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1156",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_30_33,LEG_30_124"
  },
  {
    "id": 1157,
    "start_hour": 723,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1157",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_31_123,LEG_31_95"
  },
  {
    "id": 1158,
    "start_hour": 680,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1158",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_29_38,LEG_29_157"
  },
  {
    "id": 1159,
    "start_hour": 687,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1159",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_30_142,LEG_30_139"
  },
  {
    "id": 1160,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1160",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_31_80,LEG_31_115,LEG_31_124,LEG_31_197"
  },
  {
    "id": 1161,
    "start_hour": 661,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1161",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_29_91,LEG_29_87"
  },
  {
    "id": 1162,
    "start_hour": 687,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1162",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_30_120,LEG_30_119"
  },
  {
    "id": 1163,
    "start_hour": 709,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1163",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_31_117,LEG_31_196,LEG_31_142"
  },
  {
    "id": 1164,
    "start_hour": 678,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1164",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_29_1,LEG_29_3"
  },
  {
    "id": 1165,
    "start_hour": 680,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1165",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_29_171,LEG_29_122"
  },
  {
    "id": 1166,
    "start_hour": 679,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1166",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_29_146,LEG_29_208"
  },
  {
    "id": 1167,
    "start_hour": 682,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1167",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_29_29"
  },
  {
    "id": 1168,
    "start_hour": 685,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1168",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_30_74,LEG_30_23"
  },
  {
    "id": 1169,
    "start_hour": 675,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1169",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_29_94,LEG_29_92"
  },
  {
    "id": 1170,
    "start_hour": 676,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1170",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_29_161,LEG_29_164"
  },
  {
    "id": 1171,
    "start_hour": 664,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1171",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_29_2,LEG_29_23"
  },
  {
    "id": 1172,
    "start_hour": 678,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1172",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_29_36,LEG_29_41"
  },
  {
    "id": 1173,
    "start_hour": 677,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1173",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_29_25,LEG_29_66"
  },
  {
    "id": 1174,
    "start_hour": 676,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1174",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_29_156,LEG_29_154"
  },
  {
    "id": 1175,
    "start_hour": 678,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1175",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_29_116,LEG_29_118"
  },
  {
    "id": 1176,
    "start_hour": 673,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1176",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_29_19,LEG_29_56,LEG_29_188"
  },
  {
    "id": 1177,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1177",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_30_194,LEG_30_10,LEG_30_53,LEG_30_157"
  },
  {
    "id": 1178,
    "start_hour": 709,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1178",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_31_166,LEG_31_70,LEG_31_28"
  },
  {
    "id": 1179,
    "start_hour": 673,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1179",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_29_172,LEG_29_198,LEG_29_151"
  },
  {
    "id": 1180,
    "start_hour": 697,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1180",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_30_181,LEG_30_182"
  },
  {
    "id": 1181,
    "start_hour": 720,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1181",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_31_93,LEG_31_135"
  },
  {
    "id": 1182,
    "start_hour": 674,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1182",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_29_105,LEG_29_180,LEG_29_7"
  },
  {
    "id": 1183,
    "start_hour": 698,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1183",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_30_78,LEG_30_129"
  },
  {
    "id": 1184,
    "start_hour": 721,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1184",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_31_127,LEG_31_130"
  },
  {
    "id": 1185,
    "start_hour": 266,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1185",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_12_160"
  },
  {
    "id": 1186,
    "start_hour": 253,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1186",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_12_129,LEG_12_36"
  },
  {
    "id": 1187,
    "start_hour": 265,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1187",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_12_151,LEG_12_149"
  },
  {
    "id": 1188,
    "start_hour": 253,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1188",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_12_47,LEG_12_42"
  },
  {
    "id": 1189,
    "start_hour": 265,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1189",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_12_41,LEG_12_43"
  },
  {
    "id": 1190,
    "start_hour": 272,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1190",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_12_162"
  },
  {
    "id": 1191,
    "start_hour": 265,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1191",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_12_167,LEG_12_79,LEG_12_38"
  },
  {
    "id": 1192,
    "start_hour": 288,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1192",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_13_22,LEG_13_62,LEG_13_172"
  },
  {
    "id": 1193,
    "start_hour": 267,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1193",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_12_99,LEG_12_121,LEG_12_16"
  },
  {
    "id": 1194,
    "start_hour": 287,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1194",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_13_76,LEG_13_161"
  },
  {
    "id": 1195,
    "start_hour": 320,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1195",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_14_172"
  },
  {
    "id": 1196,
    "start_hour": 275,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1196",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_12_10"
  },
  {
    "id": 1197,
    "start_hour": 292,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1197",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_13_11,LEG_13_10"
  },
  {
    "id": 1198,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1198",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_14_180,LEG_14_43,LEG_14_51"
  },
  {
    "id": 1199,
    "start_hour": 268,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1199",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_12_103,LEG_12_28"
  },
  {
    "id": 1200,
    "start_hour": 276,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1200",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_13_116,LEG_13_57,LEG_13_68"
  },
  {
    "id": 1201,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1201",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_22_1,LEG_22_5"
  },
  {
    "id": 1202,
    "start_hour": 495,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1202",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_22_3,LEG_22_24"
  },
  {
    "id": 1203,
    "start_hour": 504,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1203",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_22_168,LEG_22_169"
  },
  {
    "id": 1204,
    "start_hour": 506,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1204",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_22_97,LEG_22_101"
  },
  {
    "id": 1205,
    "start_hour": 511,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1205",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_22_19,LEG_22_18"
  },
  {
    "id": 1206,
    "start_hour": 518,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1206",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_23_129,LEG_23_128"
  },
  {
    "id": 1207,
    "start_hour": 510,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1207",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_22_124,LEG_22_93"
  },
  {
    "id": 1208,
    "start_hour": 518,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1208",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_23_139,LEG_23_161"
  },
  {
    "id": 1209,
    "start_hour": 509,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1209",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_22_149,LEG_22_158,LEG_22_27"
  },
  {
    "id": 1210,
    "start_hour": 528,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1210",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_23_171,LEG_23_21,LEG_23_16"
  },
  {
    "id": 1211,
    "start_hour": 513,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1211",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_22_94,LEG_22_71"
  },
  {
    "id": 1212,
    "start_hour": 518,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1212",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_23_25"
  },
  {
    "id": 1213,
    "start_hour": 507,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1213",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_22_147,LEG_22_146"
  },
  {
    "id": 1214,
    "start_hour": 516,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1214",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_23_87,LEG_23_96,LEG_23_172,LEG_23_88"
  },
  {
    "id": 1215,
    "start_hour": 509,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1215",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_22_6,LEG_22_2"
  },
  {
    "id": 1216,
    "start_hour": 504,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1216",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_22_15,LEG_22_38"
  },
  {
    "id": 1217,
    "start_hour": 519,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1217",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_23_70,LEG_23_162"
  },
  {
    "id": 1218,
    "start_hour": 541,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1218",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_24_177,LEG_24_32"
  },
  {
    "id": 1219,
    "start_hour": 510,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1219",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_22_120,LEG_22_79"
  },
  {
    "id": 1220,
    "start_hour": 518,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1220",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_23_121,LEG_23_119,LEG_23_155,LEG_23_150"
  },
  {
    "id": 1221,
    "start_hour": 541,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1221",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_24_8,LEG_24_107,LEG_24_163,LEG_24_77"
  },
  {
    "id": 1222,
    "start_hour": 514,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1222",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_22_100"
  },
  {
    "id": 1223,
    "start_hour": 530,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1223",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_23_99"
  },
  {
    "id": 1224,
    "start_hour": 540,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1224",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_24_87,LEG_24_96,LEG_24_147,LEG_24_146"
  },
  {
    "id": 1225,
    "start_hour": 509,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1225",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_22_85,LEG_22_86,LEG_22_145"
  },
  {
    "id": 1226,
    "start_hour": 528,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1226",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_23_126,LEG_23_193"
  },
  {
    "id": 1227,
    "start_hour": 552,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1227",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_24_195"
  },
  {
    "id": 1228,
    "start_hour": 509,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1228",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_22_118,LEG_22_115,LEG_22_157"
  },
  {
    "id": 1229,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1229",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_23_34,LEG_23_174"
  },
  {
    "id": 1230,
    "start_hour": 555,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1230",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_24_168,LEG_24_17"
  },
  {
    "id": 1231,
    "start_hour": 575,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1231",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_25_20,LEG_25_104,LEG_25_102"
  },
  {
    "id": 1232,
    "start_hour": 511,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1232",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_22_78,LEG_22_138"
  },
  {
    "id": 1233,
    "start_hour": 519,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1233",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_23_33,LEG_23_109,LEG_23_189"
  },
  {
    "id": 1234,
    "start_hour": 553,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1234",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_24_117,LEG_24_193"
  },
  {
    "id": 1235,
    "start_hour": 576,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1235",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_25_195"
  },
  {
    "id": 1236,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1236",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_11_1,LEG_11_5"
  },
  {
    "id": 1237,
    "start_hour": 243,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1237",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_11_30,LEG_11_29"
  },
  {
    "id": 1238,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1238",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_11_95,LEG_11_99"
  },
  {
    "id": 1239,
    "start_hour": 245,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1239",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_11_145,LEG_11_154"
  },
  {
    "id": 1240,
    "start_hour": 243,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1240",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_11_143,LEG_11_142"
  },
  {
    "id": 1241,
    "start_hour": 250,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1241",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_11_98"
  },
  {
    "id": 1242,
    "start_hour": 266,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1242",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_12_89"
  },
  {
    "id": 1243,
    "start_hour": 240,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1243",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_11_14,LEG_11_36,LEG_11_73"
  },
  {
    "id": 1244,
    "start_hour": 263,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1244",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_12_66,LEG_12_30,LEG_12_139"
  },
  {
    "id": 1245,
    "start_hour": 249,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1245",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_11_92,LEG_11_69"
  },
  {
    "id": 1246,
    "start_hour": 254,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1246",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_12_22"
  },
  {
    "id": 1247,
    "start_hour": 240,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1247",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_11_165,LEG_11_177,LEG_11_46"
  },
  {
    "id": 1248,
    "start_hour": 264,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1248",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_12_6"
  },
  {
    "id": 1249,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1249",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_11_88,LEG_11_126"
  },
  {
    "id": 1250,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1250",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_11_120,LEG_11_91"
  },
  {
    "id": 1251,
    "start_hour": 245,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1251",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_11_6,LEG_11_2"
  },
  {
    "id": 1252,
    "start_hour": 247,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1252",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_11_18,LEG_11_17"
  },
  {
    "id": 1253,
    "start_hour": 246,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1253",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_11_116,LEG_11_77"
  },
  {
    "id": 1254,
    "start_hour": 228,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1254",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_11_123,LEG_11_113,LEG_11_171"
  },
  {
    "id": 1255,
    "start_hour": 265,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1255",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_12_106,LEG_12_176"
  },
  {
    "id": 1256,
    "start_hour": 288,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1256",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_13_176"
  },
  {
    "id": 1257,
    "start_hour": 252,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1257",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_11_28"
  },
  {
    "id": 1258,
    "start_hour": 264,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1258",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_12_75,LEG_12_124"
  },
  {
    "id": 1259,
    "start_hour": 289,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1259",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_13_126,LEG_13_78,LEG_13_79"
  },
  {
    "id": 1260,
    "start_hour": 252,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1260",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_11_21"
  },
  {
    "id": 1261,
    "start_hour": 265,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1261",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_12_134,LEG_12_107,LEG_12_104,LEG_12_105"
  },
  {
    "id": 1262,
    "start_hour": 288,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1262",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_13_95,LEG_13_146,LEG_13_147"
  },
  {
    "id": 1263,
    "start_hour": 248,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1263",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_11_151,LEG_11_146"
  },
  {
    "id": 1264,
    "start_hour": 254,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1264",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_12_118,LEG_12_117,LEG_12_143"
  },
  {
    "id": 1265,
    "start_hour": 289,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1265",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_13_33,LEG_13_174"
  },
  {
    "id": 1266,
    "start_hour": 312,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1266",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_14_188"
  },
  {
    "id": 1267,
    "start_hour": 250,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1267",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_11_127"
  },
  {
    "id": 1268,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1268",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_12_112,LEG_12_158"
  },
  {
    "id": 1269,
    "start_hour": 289,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1269",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_13_106,LEG_13_90,LEG_13_157,LEG_13_105"
  },
  {
    "id": 1270,
    "start_hour": 312,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1270",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_14_102,LEG_14_154,LEG_14_155"
  },
  {
    "id": 1271,
    "start_hour": 252,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1271",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_11_4"
  },
  {
    "id": 1272,
    "start_hour": 266,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1272",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_12_130,LEG_12_51,LEG_12_120"
  },
  {
    "id": 1273,
    "start_hour": 290,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1273",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_13_112"
  },
  {
    "id": 1274,
    "start_hour": 300,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1274",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_14_13,LEG_14_0,LEG_14_131,LEG_14_24"
  },
  {
    "id": 1275,
    "start_hour": 252,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1275",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_11_190"
  },
  {
    "id": 1276,
    "start_hour": 274,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1276",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_12_179"
  },
  {
    "id": 1277,
    "start_hour": 278,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1277",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_13_118,LEG_13_117,LEG_13_3"
  },
  {
    "id": 1278,
    "start_hour": 312,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1278",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_14_7"
  },
  {
    "id": 1279,
    "start_hour": 248,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1279",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_11_27,LEG_11_58"
  },
  {
    "id": 1280,
    "start_hour": 255,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1280",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_12_64"
  },
  {
    "id": 1281,
    "start_hour": 289,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1281",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_13_94,LEG_13_113,LEG_13_85"
  },
  {
    "id": 1282,
    "start_hour": 252,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1282",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_11_153"
  },
  {
    "id": 1283,
    "start_hour": 265,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1283",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_12_29,LEG_12_114"
  },
  {
    "id": 1284,
    "start_hour": 288,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1284",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_13_122,LEG_13_124"
  },
  {
    "id": 1285,
    "start_hour": 313,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1285",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_14_132,LEG_14_115,LEG_14_79"
  },
  {
    "id": 1286,
    "start_hour": 543,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1286",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_24_3,LEG_24_24"
  },
  {
    "id": 1287,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1287",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_24_1,LEG_24_5"
  },
  {
    "id": 1288,
    "start_hour": 552,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1288",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_24_169,LEG_24_170"
  },
  {
    "id": 1289,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1289",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_24_97,LEG_24_101"
  },
  {
    "id": 1290,
    "start_hour": 556,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1290",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_24_136,LEG_24_26"
  },
  {
    "id": 1291,
    "start_hour": 565,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1291",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_25_8,LEG_25_107,LEG_25_163,LEG_25_77"
  },
  {
    "id": 1292,
    "start_hour": 561,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1292",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_24_94,LEG_24_71"
  },
  {
    "id": 1293,
    "start_hour": 566,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1293",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_25_25"
  },
  {
    "id": 1294,
    "start_hour": 555,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1294",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_24_31,LEG_24_30,LEG_24_27"
  },
  {
    "id": 1295,
    "start_hour": 576,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1295",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_25_171,LEG_25_31,LEG_25_30"
  },
  {
    "id": 1296,
    "start_hour": 557,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1296",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_24_149,LEG_24_158,LEG_24_29"
  },
  {
    "id": 1297,
    "start_hour": 576,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1297",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_25_84,LEG_25_98,LEG_25_173"
  },
  {
    "id": 1298,
    "start_hour": 557,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1298",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_24_6,LEG_24_2"
  },
  {
    "id": 1299,
    "start_hour": 542,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1299",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_24_121,LEG_24_119,LEG_24_78,LEG_24_138"
  },
  {
    "id": 1300,
    "start_hour": 540,
    "duration_hours": 27,
    "required_skill": "A321",
    "gerad_duty_id": "D1300",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_24_127,LEG_24_66,LEG_24_76,LEG_24_23,LEG_24_192"
  },
  {
    "id": 1301,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1301",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_25_185,LEG_25_110,LEG_25_14,LEG_25_116"
  },
  {
    "id": 1302,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1302",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_26_96,LEG_26_149,LEG_26_150"
  },
  {
    "id": 1303,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1303",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_24_98,LEG_24_173,LEG_24_154"
  },
  {
    "id": 1304,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1304",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_25_152,LEG_25_174"
  },
  {
    "id": 1305,
    "start_hour": 601,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1305",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_26_111,LEG_26_129"
  },
  {
    "id": 1306,
    "start_hour": 552,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1306",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_24_15,LEG_24_38"
  },
  {
    "id": 1307,
    "start_hour": 567,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1307",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_25_70,LEG_25_162"
  },
  {
    "id": 1308,
    "start_hour": 589,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1308",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_26_164,LEG_26_28"
  },
  {
    "id": 1309,
    "start_hour": 562,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1309",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_24_100"
  },
  {
    "id": 1310,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1310",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_25_99,LEG_25_131"
  },
  {
    "id": 1311,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1311",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_26_117,LEG_26_41"
  },
  {
    "id": 1312,
    "start_hour": 625,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1312",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_27_61,LEG_27_38,LEG_27_24"
  },
  {
    "id": 1313,
    "start_hour": 542,
    "duration_hours": 18,
    "required_skill": "A321",
    "gerad_duty_id": "D1313",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_24_151,LEG_24_156"
  },
  {
    "id": 1314,
    "start_hour": 564,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1314",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_25_127,LEG_25_66,LEG_25_76,LEG_25_23"
  },
  {
    "id": 1315,
    "start_hour": 624,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1315",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_27_130,LEG_27_53,LEG_27_31"
  },
  {
    "id": 1316,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1316",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_24_90,LEG_24_130"
  },
  {
    "id": 1317,
    "start_hour": 566,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1317",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_25_139,LEG_25_161,LEG_25_22"
  },
  {
    "id": 1318,
    "start_hour": 601,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1318",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_26_139,LEG_26_84,LEG_26_124"
  },
  {
    "id": 1319,
    "start_hour": 559,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1319",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_24_19,LEG_24_18"
  },
  {
    "id": 1320,
    "start_hour": 566,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1320",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_25_129,LEG_25_128,LEG_25_157"
  },
  {
    "id": 1321,
    "start_hour": 601,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1321",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_26_31,LEG_26_114,LEG_26_76"
  },
  {
    "id": 1322,
    "start_hour": 560,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1322",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_24_28,LEG_24_60"
  },
  {
    "id": 1323,
    "start_hour": 601,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1323",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_26_94,LEG_26_118,LEG_26_87"
  },
  {
    "id": 1324,
    "start_hour": 560,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1324",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_24_155,LEG_24_150"
  },
  {
    "id": 1325,
    "start_hour": 566,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1325",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_25_151,LEG_25_156,LEG_25_194"
  },
  {
    "id": 1326,
    "start_hour": 610,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1326",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_26_181"
  },
  {
    "id": 1327,
    "start_hour": 542,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1327",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_24_129,LEG_24_128,LEG_24_157"
  },
  {
    "id": 1328,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1328",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_25_34,LEG_25_154"
  },
  {
    "id": 1329,
    "start_hour": 602,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1329",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_26_143,LEG_26_178"
  },
  {
    "id": 1330,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1330",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_24_124,LEG_24_93"
  },
  {
    "id": 1331,
    "start_hour": 567,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1331",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_25_33,LEG_25_109,LEG_25_189"
  },
  {
    "id": 1332,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1332",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_26_60,LEG_26_69,LEG_26_174,LEG_26_53"
  },
  {
    "id": 1333,
    "start_hour": 626,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1333",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_27_90,LEG_27_98,LEG_27_155"
  },
  {
    "id": 1334,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1334",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_05_1,LEG_05_4"
  },
  {
    "id": 1335,
    "start_hour": 101,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1335",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_05_78,LEG_05_79"
  },
  {
    "id": 1336,
    "start_hour": 101,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1336",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_05_141,LEG_05_150"
  },
  {
    "id": 1337,
    "start_hour": 104,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1337",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_05_23,LEG_05_55"
  },
  {
    "id": 1338,
    "start_hour": 110,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1338",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_06_21"
  },
  {
    "id": 1339,
    "start_hour": 96,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1339",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_05_17,LEG_05_33,LEG_05_69"
  },
  {
    "id": 1340,
    "start_hour": 119,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1340",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_06_62,LEG_06_31,LEG_06_135"
  },
  {
    "id": 1341,
    "start_hour": 101,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1341",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_05_5,LEG_05_2"
  },
  {
    "id": 1342,
    "start_hour": 102,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1342",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_05_119,LEG_05_86"
  },
  {
    "id": 1343,
    "start_hour": 96,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1343",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_05_159,LEG_05_171,LEG_05_44"
  },
  {
    "id": 1344,
    "start_hour": 120,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1344",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_06_8,LEG_06_171"
  },
  {
    "id": 1345,
    "start_hour": 144,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1345",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_07_190"
  },
  {
    "id": 1346,
    "start_hour": 97,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1346",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_05_71,LEG_05_103,LEG_05_15,LEG_05_105,LEG_05_98"
  },
  {
    "id": 1347,
    "start_hour": 120,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1347",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_06_92,LEG_06_159,LEG_06_36,LEG_06_64"
  },
  {
    "id": 1348,
    "start_hour": 143,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1348",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_07_71,LEG_07_102,LEG_07_165"
  },
  {
    "id": 1349,
    "start_hour": 108,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1349",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_05_149"
  },
  {
    "id": 1350,
    "start_hour": 121,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1350",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_06_30,LEG_06_85,LEG_06_152,LEG_06_15"
  },
  {
    "id": 1351,
    "start_hour": 143,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1351",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_07_19,LEG_07_101,LEG_07_100"
  },
  {
    "id": 1352,
    "start_hour": 106,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1352",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_05_92"
  },
  {
    "id": 1353,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1353",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_06_86,LEG_06_153"
  },
  {
    "id": 1354,
    "start_hour": 145,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1354",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_07_112,LEG_07_113,LEG_07_110"
  },
  {
    "id": 1355,
    "start_hour": 97,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1355",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_05_7,LEG_05_64,LEG_05_53"
  },
  {
    "id": 1356,
    "start_hour": 123,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1356",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_06_53,LEG_06_65,LEG_06_20,LEG_06_170"
  },
  {
    "id": 1357,
    "start_hour": 147,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1357",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_07_163,LEG_07_150,LEG_07_145"
  },
  {
    "id": 1358,
    "start_hour": 157,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1358",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_08_8,LEG_08_105,LEG_08_159,LEG_08_75"
  },
  {
    "id": 1359,
    "start_hour": 106,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1359",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_05_126"
  },
  {
    "id": 1360,
    "start_hour": 122,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1360",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_06_106,LEG_06_87"
  },
  {
    "id": 1361,
    "start_hour": 146,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1361",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_07_97"
  },
  {
    "id": 1362,
    "start_hour": 156,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1362",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_08_13,LEG_08_0,LEG_08_132,LEG_08_25"
  },
  {
    "id": 1363,
    "start_hour": 106,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1363",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_05_151"
  },
  {
    "id": 1364,
    "start_hour": 109,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1364",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_06_72"
  },
  {
    "id": 1365,
    "start_hour": 144,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1365",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_07_78,LEG_07_139,LEG_07_55,LEG_07_46"
  },
  {
    "id": 1366,
    "start_hour": 168,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1366",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_08_7"
  },
  {
    "id": 1367,
    "start_hour": 103,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1367",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_05_165"
  },
  {
    "id": 1368,
    "start_hour": 110,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1368",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_06_133,LEG_06_138,LEG_06_136"
  },
  {
    "id": 1369,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1369",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_07_147,LEG_07_169"
  },
  {
    "id": 1370,
    "start_hour": 169,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1370",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_08_113"
  },
  {
    "id": 1371,
    "start_hour": 104,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1371",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_05_147,LEG_05_142"
  },
  {
    "id": 1372,
    "start_hour": 110,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1372",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_06_79,LEG_06_82,LEG_06_114"
  },
  {
    "id": 1373,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1373",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_07_118,LEG_07_16"
  },
  {
    "id": 1374,
    "start_hour": 167,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1374",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_08_19,LEG_08_102,LEG_08_100"
  },
  {
    "id": 1375,
    "start_hour": 108,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1375",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_05_183"
  },
  {
    "id": 1376,
    "start_hour": 130,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1376",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_06_174"
  },
  {
    "id": 1377,
    "start_hour": 134,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1377",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_07_116,LEG_07_114,LEG_07_76,LEG_07_133"
  },
  {
    "id": 1378,
    "start_hour": 108,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1378",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_05_24"
  },
  {
    "id": 1379,
    "start_hour": 120,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1379",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_06_74,LEG_06_118"
  },
  {
    "id": 1380,
    "start_hour": 145,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1380",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_07_132,LEG_07_130"
  },
  {
    "id": 1381,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1381",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_08_133,LEG_08_116,LEG_08_77"
  },
  {
    "id": 1382,
    "start_hour": 108,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1382",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_05_3"
  },
  {
    "id": 1383,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1383",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_06_126,LEG_06_51,LEG_06_50"
  },
  {
    "id": 1384,
    "start_hour": 147,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1384",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_07_59,LEG_07_74,LEG_07_22,LEG_07_51"
  },
  {
    "id": 1385,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1385",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_08_144,LEG_08_88,LEG_08_126"
  },
  {
    "id": 1386,
    "start_hour": 374,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1386",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_17_36,LEG_17_106"
  },
  {
    "id": 1387,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1387",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_17_63,LEG_17_111,LEG_17_11"
  },
  {
    "id": 1388,
    "start_hour": 407,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1388",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_18_166,LEG_18_41,LEG_18_64"
  },
  {
    "id": 1389,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1389",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_17_67,LEG_17_68"
  },
  {
    "id": 1390,
    "start_hour": 390,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1390",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_17_76,LEG_17_23,LEG_17_53"
  },
  {
    "id": 1391,
    "start_hour": 410,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1391",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_18_43,LEG_18_192"
  },
  {
    "id": 1392,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1392",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_19_177,LEG_19_176,LEG_19_9"
  },
  {
    "id": 1393,
    "start_hour": 462,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1393",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_20_26"
  },
  {
    "id": 1394,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1394",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_17_65,LEG_17_122"
  },
  {
    "id": 1395,
    "start_hour": 399,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1395",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_18_33,LEG_18_61,LEG_18_58"
  },
  {
    "id": 1396,
    "start_hour": 435,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1396",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_19_57,LEG_19_35,LEG_19_33"
  },
  {
    "id": 1397,
    "start_hour": 386,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1397",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_17_35,LEG_17_153"
  },
  {
    "id": 1398,
    "start_hour": 396,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1398",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_18_13,LEG_18_0,LEG_18_104,LEG_18_102,LEG_18_193"
  },
  {
    "id": 1399,
    "start_hour": 432,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1399",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_19_186,LEG_19_88"
  },
  {
    "id": 1400,
    "start_hour": 460,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1400",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_20_35,LEG_20_36,LEG_20_34"
  },
  {
    "id": 1401,
    "start_hour": 361,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1401",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_16_45,LEG_16_47"
  },
  {
    "id": 1402,
    "start_hour": 361,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1402",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_16_167,LEG_16_164"
  },
  {
    "id": 1403,
    "start_hour": 349,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1403",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_16_54,LEG_16_46"
  },
  {
    "id": 1404,
    "start_hour": 349,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1404",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_16_141,LEG_16_42"
  },
  {
    "id": 1405,
    "start_hour": 362,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1405",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_16_176"
  },
  {
    "id": 1406,
    "start_hour": 392,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1406",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_17_178"
  },
  {
    "id": 1407,
    "start_hour": 368,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1407",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_16_113,LEG_16_114"
  },
  {
    "id": 1408,
    "start_hour": 361,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1408",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_16_183,LEG_16_181,LEG_16_165,LEG_16_82"
  },
  {
    "id": 1409,
    "start_hour": 368,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1409",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_16_56,LEG_16_51"
  },
  {
    "id": 1410,
    "start_hour": 371,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1410",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_16_10"
  },
  {
    "id": 1411,
    "start_hour": 388,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1411",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_17_12"
  },
  {
    "id": 1412,
    "start_hour": 398,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1412",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_18_191,LEG_18_186,LEG_18_50,LEG_18_55"
  },
  {
    "id": 1413,
    "start_hour": 363,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1413",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_16_108,LEG_16_132,LEG_16_4"
  },
  {
    "id": 1414,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1414",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_17_143,LEG_17_81,LEG_17_112,LEG_17_165,LEG_17_82"
  },
  {
    "id": 1415,
    "start_hour": 369,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1415",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_16_140,LEG_16_142"
  },
  {
    "id": 1416,
    "start_hour": 373,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1416",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_17_83"
  },
  {
    "id": 1417,
    "start_hour": 408,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1417",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_18_80,LEG_18_144,LEG_18_57,LEG_18_11"
  },
  {
    "id": 1418,
    "start_hour": 431,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1418",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_19_158,LEG_19_159,LEG_19_157"
  },
  {
    "id": 1419,
    "start_hour": 629,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1419",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_27_152,LEG_27_164,LEG_27_163"
  },
  {
    "id": 1420,
    "start_hour": 652,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1420",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_28_198,LEG_28_72"
  },
  {
    "id": 1421,
    "start_hour": 676,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1421",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_29_67,LEG_29_68,LEG_29_51"
  },
  {
    "id": 1422,
    "start_hour": 633,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1422",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_27_15"
  },
  {
    "id": 1423,
    "start_hour": 666,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1423",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_29_16,LEG_29_11,LEG_29_9"
  },
  {
    "id": 1424,
    "start_hour": 627,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1424",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_27_140"
  },
  {
    "id": 1425,
    "start_hour": 656,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1425",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_28_175"
  },
  {
    "id": 1426,
    "start_hour": 626,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1426",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_27_149,LEG_27_148,LEG_27_79"
  },
  {
    "id": 1427,
    "start_hour": 636,
    "duration_hours": 2,
    "required_skill": "A320",
    "gerad_duty_id": "D1427",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_28_163"
  },
  {
    "id": 1428,
    "start_hour": 633,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1428",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_27_47"
  },
  {
    "id": 1429,
    "start_hour": 637,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1429",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_28_48"
  },
  {
    "id": 1430,
    "start_hour": 628,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1430",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_27_126,LEG_27_100"
  },
  {
    "id": 1431,
    "start_hour": 637,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1431",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_28_79,LEG_28_59,LEG_28_60,LEG_28_6"
  },
  {
    "id": 1432,
    "start_hour": 628,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1432",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_27_81,LEG_27_138"
  },
  {
    "id": 1433,
    "start_hour": 637,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1433",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_28_115,LEG_28_150,LEG_28_165,LEG_28_183"
  },
  {
    "id": 1434,
    "start_hour": 613,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1434",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_27_105,LEG_27_109"
  },
  {
    "id": 1435,
    "start_hour": 626,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1435",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_27_108,LEG_27_80"
  },
  {
    "id": 1436,
    "start_hour": 630,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1436",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_27_49,LEG_27_45"
  },
  {
    "id": 1437,
    "start_hour": 699,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1437",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_30_42,LEG_30_25"
  },
  {
    "id": 1438,
    "start_hour": 709,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1438",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_31_119,LEG_31_155,LEG_31_169,LEG_31_207"
  },
  {
    "id": 1439,
    "start_hour": 699,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1439",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_30_149,LEG_30_12,LEG_30_7"
  },
  {
    "id": 1440,
    "start_hour": 723,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1440",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_31_60,LEG_31_194"
  },
  {
    "id": 1441,
    "start_hour": 687,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1441",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_30_43,LEG_30_20"
  },
  {
    "id": 1442,
    "start_hour": 721,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1442",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_31_19,LEG_31_46"
  },
  {
    "id": 1443,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1443",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_30_98,LEG_30_188,LEG_30_56"
  },
  {
    "id": 1444,
    "start_hour": 720,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1444",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_31_103,LEG_31_14,LEG_31_152"
  },
  {
    "id": 1445,
    "start_hour": 687,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1445",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_30_70,LEG_30_68"
  },
  {
    "id": 1446,
    "start_hour": 688,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1446",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_30_164,LEG_30_159"
  },
  {
    "id": 1447,
    "start_hour": 710,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1447",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_31_167,LEG_31_100"
  },
  {
    "id": 1448,
    "start_hour": 687,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1448",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_30_146,LEG_30_204"
  },
  {
    "id": 1449,
    "start_hour": 74,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1449",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_04_172"
  },
  {
    "id": 1450,
    "start_hour": 61,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1450",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_04_52,LEG_04_44"
  },
  {
    "id": 1451,
    "start_hour": 61,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1451",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_04_137,LEG_04_40"
  },
  {
    "id": 1452,
    "start_hour": 73,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1452",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_04_163,LEG_04_160,LEG_04_178"
  },
  {
    "id": 1453,
    "start_hour": 97,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1453",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_05_45,LEG_05_46,LEG_05_49"
  },
  {
    "id": 1454,
    "start_hour": 80,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1454",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_04_54,LEG_04_49"
  },
  {
    "id": 1455,
    "start_hour": 73,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1455",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_04_179,LEG_04_177,LEG_04_161,LEG_04_80"
  },
  {
    "id": 1456,
    "start_hour": 80,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1456",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_04_109,LEG_04_110"
  },
  {
    "id": 1457,
    "start_hour": 75,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1457",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_04_106,LEG_04_128,LEG_04_21"
  },
  {
    "id": 1458,
    "start_hour": 97,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1458",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_05_140,LEG_05_83,LEG_05_125"
  },
  {
    "id": 1459,
    "start_hour": 123,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1459",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_06_18,LEG_06_41"
  },
  {
    "id": 1460,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1460",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_04_43,LEG_04_15,LEG_04_170"
  },
  {
    "id": 1461,
    "start_hour": 97,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1461",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_05_112,LEG_05_90,LEG_05_163,LEG_05_38"
  },
  {
    "id": 1462,
    "start_hour": 120,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1462",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_06_22,LEG_06_61,LEG_06_167"
  },
  {
    "id": 1463,
    "start_hour": 83,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1463",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_04_10"
  },
  {
    "id": 1464,
    "start_hour": 100,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1464",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_05_12,LEG_05_51"
  },
  {
    "id": 1465,
    "start_hour": 126,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1465",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_06_14"
  },
  {
    "id": 1466,
    "start_hour": 132,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1466",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_07_122,LEG_07_180,LEG_07_179,LEG_07_9"
  },
  {
    "id": 1467,
    "start_hour": 81,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1467",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_04_136,LEG_04_138"
  },
  {
    "id": 1468,
    "start_hour": 85,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1468",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_05_76"
  },
  {
    "id": 1469,
    "start_hour": 120,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1469",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_06_69,LEG_06_127,LEG_06_49,LEG_06_161"
  },
  {
    "id": 1470,
    "start_hour": 145,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1470",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_07_47,LEG_07_48,LEG_07_53"
  },
  {
    "id": 1471,
    "start_hour": 457,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1471",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_20_75,LEG_20_171,LEG_20_8"
  },
  {
    "id": 1472,
    "start_hour": 445,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1472",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_20_49,LEG_20_41"
  },
  {
    "id": 1473,
    "start_hour": 457,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1473",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_20_155,LEG_20_153"
  },
  {
    "id": 1474,
    "start_hour": 457,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1474",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_20_170,LEG_20_176"
  },
  {
    "id": 1475,
    "start_hour": 445,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1475",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_20_132,LEG_20_38"
  },
  {
    "id": 1476,
    "start_hour": 467,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1476",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_20_48"
  },
  {
    "id": 1477,
    "start_hour": 482,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1477",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_21_43,LEG_21_184,LEG_21_9"
  },
  {
    "id": 1478,
    "start_hour": 462,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1478",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_20_101,LEG_20_12,LEG_20_24"
  },
  {
    "id": 1479,
    "start_hour": 480,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1479",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_21_171,LEG_21_21,LEG_21_47"
  },
  {
    "id": 1480,
    "start_hour": 464,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1480",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_20_51,LEG_20_46"
  },
  {
    "id": 1481,
    "start_hour": 467,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1481",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_20_9"
  },
  {
    "id": 1482,
    "start_hour": 484,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1482",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_21_12,LEG_21_53"
  },
  {
    "id": 1483,
    "start_hour": 506,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1483",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_22_43,LEG_22_44,LEG_22_52"
  },
  {
    "id": 1484,
    "start_hour": 445,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1484",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_20_165,LEG_20_30"
  },
  {
    "id": 1485,
    "start_hour": 470,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D1485",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_21_92,LEG_21_95"
  },
  {
    "id": 1486,
    "start_hour": 492,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D1486",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_22_127,LEG_22_184,LEG_22_183,LEG_22_9"
  },
  {
    "id": 1487,
    "start_hour": 459,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1487",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_20_100,LEG_20_123,LEG_20_82"
  },
  {
    "id": 1488,
    "start_hour": 482,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1488",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_21_91,LEG_21_175"
  },
  {
    "id": 1489,
    "start_hour": 504,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1489",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_22_62,LEG_22_72,LEG_22_187"
  },
  {
    "id": 1490,
    "start_hour": 98,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1490",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_05_166"
  },
  {
    "id": 1491,
    "start_hour": 95,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1491",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_05_99,LEG_05_108,LEG_05_75,LEG_05_107"
  },
  {
    "id": 1492,
    "start_hour": 85,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1492",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_05_135,LEG_05_36"
  },
  {
    "id": 1493,
    "start_hour": 85,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1493",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_05_48,LEG_05_42"
  },
  {
    "id": 1494,
    "start_hour": 104,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1494",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_05_168"
  },
  {
    "id": 1495,
    "start_hour": 107,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1495",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_05_10"
  },
  {
    "id": 1496,
    "start_hour": 124,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1496",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_06_13"
  },
  {
    "id": 1497,
    "start_hour": 97,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1497",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_05_157,LEG_05_155,LEG_05_172"
  },
  {
    "id": 1498,
    "start_hour": 121,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1498",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_06_43,LEG_06_166"
  },
  {
    "id": 1499,
    "start_hour": 144,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1499",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_07_60,LEG_07_70,LEG_07_183"
  },
  {
    "id": 1500,
    "start_hour": 97,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1500",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_05_41,LEG_05_43,LEG_05_11"
  },
  {
    "id": 1501,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1501",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_06_165,LEG_06_70,LEG_06_96,LEG_06_168"
  },
  {
    "id": 1502,
    "start_hour": 144,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1502",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_07_174,LEG_07_42,LEG_07_50"
  },
  {
    "id": 1503,
    "start_hour": 100,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1503",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_05_109,LEG_05_28,LEG_05_16"
  },
  {
    "id": 1504,
    "start_hour": 119,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1504",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_06_73,LEG_06_156"
  },
  {
    "id": 1505,
    "start_hour": 152,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1505",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_07_173"
  },
  {
    "id": 1506,
    "start_hour": 158,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1506",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_08_187,LEG_08_182,LEG_08_48,LEG_08_53"
  },
  {
    "id": 1507,
    "start_hour": 104,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1507",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_05_52"
  },
  {
    "id": 1508,
    "start_hour": 108,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1508",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_06_110,LEG_06_100,LEG_06_103,LEG_06_68"
  },
  {
    "id": 1509,
    "start_hour": 144,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1509",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_07_164,LEG_07_57,LEG_07_73"
  },
  {
    "id": 1510,
    "start_hour": 167,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1510",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_08_71,LEG_08_70,LEG_08_184"
  },
  {
    "id": 1511,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1511",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_20_90,LEG_20_94"
  },
  {
    "id": 1512,
    "start_hour": 460,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1512",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_20_127,LEG_20_23"
  },
  {
    "id": 1513,
    "start_hour": 459,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1513",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_20_138,LEG_20_137"
  },
  {
    "id": 1514,
    "start_hour": 448,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1514",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_20_0,LEG_20_89"
  },
  {
    "id": 1515,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1515",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_20_1,LEG_20_4"
  },
  {
    "id": 1516,
    "start_hour": 459,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1516",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_20_29,LEG_20_28"
  },
  {
    "id": 1517,
    "start_hour": 456,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1517",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_20_157,LEG_20_58,LEG_20_66"
  },
  {
    "id": 1518,
    "start_hour": 479,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1518",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_21_73,LEG_21_35,LEG_21_153"
  },
  {
    "id": 1519,
    "start_hour": 465,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1519",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_20_87,LEG_20_62"
  },
  {
    "id": 1520,
    "start_hour": 470,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1520",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_21_25"
  },
  {
    "id": 1521,
    "start_hour": 459,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1521",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_20_159,LEG_20_81,LEG_20_107"
  },
  {
    "id": 1522,
    "start_hour": 480,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1522",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_21_103,LEG_21_159,LEG_21_160"
  },
  {
    "id": 1523,
    "start_hour": 461,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1523",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_20_5,LEG_20_2"
  },
  {
    "id": 1524,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1524",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_20_115,LEG_20_86"
  },
  {
    "id": 1525,
    "start_hour": 463,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1525",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_20_17,LEG_20_16"
  },
  {
    "id": 1526,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1526",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_20_83,LEG_20_121"
  },
  {
    "id": 1527,
    "start_hour": 462,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1527",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_20_111,LEG_20_70"
  },
  {
    "id": 1528,
    "start_hour": 468,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1528",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_20_19"
  },
  {
    "id": 1529,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1529",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_21_148,LEG_21_118,LEG_21_115,LEG_21_116"
  },
  {
    "id": 1530,
    "start_hour": 504,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1530",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_22_103,LEG_22_159,LEG_22_160"
  },
  {
    "id": 1531,
    "start_hour": 466,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1531",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_20_122"
  },
  {
    "id": 1532,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1532",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_21_123,LEG_21_174"
  },
  {
    "id": 1533,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1533",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_22_117,LEG_22_98,LEG_22_172,LEG_22_116"
  },
  {
    "id": 1534,
    "start_hour": 528,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1534",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_23_103,LEG_23_159,LEG_23_160"
  },
  {
    "id": 1535,
    "start_hour": 468,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1535",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_20_147"
  },
  {
    "id": 1536,
    "start_hour": 481,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1536",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_21_34,LEG_21_125"
  },
  {
    "id": 1537,
    "start_hour": 504,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1537",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_22_133,LEG_22_135"
  },
  {
    "id": 1538,
    "start_hour": 529,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1538",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_23_137,LEG_23_85,LEG_23_86"
  },
  {
    "id": 1539,
    "start_hour": 464,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1539",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_20_145,LEG_20_140"
  },
  {
    "id": 1540,
    "start_hour": 471,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1540",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_21_33,LEG_21_61,LEG_21_58"
  },
  {
    "id": 1541,
    "start_hour": 507,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1541",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_22_61,LEG_22_75"
  },
  {
    "id": 1542,
    "start_hour": 527,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1542",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_23_73,LEG_23_35,LEG_23_153"
  },
  {
    "id": 1543,
    "start_hour": 466,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1543",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_20_93"
  },
  {
    "id": 1544,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1544",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_21_99,LEG_21_154"
  },
  {
    "id": 1545,
    "start_hour": 506,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1545",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_22_152"
  },
  {
    "id": 1546,
    "start_hour": 516,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1546",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_23_13,LEG_23_0,LEG_23_136,LEG_23_26"
  },
  {
    "id": 1547,
    "start_hour": 460,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1547",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_20_125,LEG_20_65"
  },
  {
    "id": 1548,
    "start_hour": 470,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1548",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_21_69,LEG_21_185,LEG_21_189"
  },
  {
    "id": 1549,
    "start_hour": 504,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1549",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_22_178,LEG_22_144,LEG_22_57,LEG_22_48"
  },
  {
    "id": 1550,
    "start_hour": 528,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1550",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_23_7"
  },
  {
    "id": 1551,
    "start_hour": 459,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1551",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_20_18,LEG_20_14,LEG_20_3"
  },
  {
    "id": 1552,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1552",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_21_143,LEG_21_81,LEG_21_112,LEG_21_182"
  },
  {
    "id": 1553,
    "start_hour": 505,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1553",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_22_49,LEG_22_81,LEG_22_112,LEG_22_188"
  },
  {
    "id": 1554,
    "start_hour": 528,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1554",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_23_62,LEG_23_105,LEG_23_170"
  },
  {
    "id": 1555,
    "start_hour": 468,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1555",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_20_179"
  },
  {
    "id": 1556,
    "start_hour": 490,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1556",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_21_196"
  },
  {
    "id": 1557,
    "start_hour": 494,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1557",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_22_151,LEG_22_156,LEG_22_22"
  },
  {
    "id": 1558,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1558",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_23_148,LEG_23_90,LEG_23_130"
  },
  {
    "id": 1559,
    "start_hour": 205,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1559",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_10_52,LEG_10_44"
  },
  {
    "id": 1560,
    "start_hour": 205,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1560",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_10_137,LEG_10_40"
  },
  {
    "id": 1561,
    "start_hour": 217,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1561",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_10_163,LEG_10_160"
  },
  {
    "id": 1562,
    "start_hour": 222,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1562",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_10_180,LEG_10_9,LEG_10_51"
  },
  {
    "id": 1563,
    "start_hour": 242,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1563",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_11_41,LEG_11_42,LEG_11_50"
  },
  {
    "id": 1564,
    "start_hour": 218,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1564",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_10_172"
  },
  {
    "id": 1565,
    "start_hour": 248,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1565",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_11_174"
  },
  {
    "id": 1566,
    "start_hour": 224,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1566",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_10_54,LEG_10_49"
  },
  {
    "id": 1567,
    "start_hour": 224,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1567",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_10_109,LEG_10_110"
  },
  {
    "id": 1568,
    "start_hour": 217,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1568",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_10_43,LEG_10_45,LEG_10_136,LEG_10_138"
  },
  {
    "id": 1569,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1569",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_10_179,LEG_10_57,LEG_10_56"
  },
  {
    "id": 1570,
    "start_hour": 243,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1570",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_11_59,LEG_11_74,LEG_11_22"
  },
  {
    "id": 1571,
    "start_hour": 254,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1571",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_12_174,LEG_12_170,LEG_12_45,LEG_12_48"
  },
  {
    "id": 1572,
    "start_hour": 219,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1572",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_10_106,LEG_10_128,LEG_10_4"
  },
  {
    "id": 1573,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1573",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_11_139,LEG_11_79,LEG_11_108,LEG_11_161,LEG_11_80"
  },
  {
    "id": 1574,
    "start_hour": 227,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1574",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_10_10"
  },
  {
    "id": 1575,
    "start_hour": 244,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1575",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_11_12,LEG_11_51"
  },
  {
    "id": 1576,
    "start_hour": 266,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1576",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_12_39,LEG_12_166"
  },
  {
    "id": 1577,
    "start_hour": 289,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1577",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_13_45,LEG_13_46,LEG_13_49"
  },
  {
    "id": 1578,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1578",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_08_95,LEG_08_99"
  },
  {
    "id": 1579,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1579",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_08_1,LEG_08_5"
  },
  {
    "id": 1580,
    "start_hour": 173,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1580",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_08_145,LEG_08_154"
  },
  {
    "id": 1581,
    "start_hour": 171,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1581",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_08_30,LEG_08_29"
  },
  {
    "id": 1582,
    "start_hour": 168,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1582",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_08_165,LEG_08_166"
  },
  {
    "id": 1583,
    "start_hour": 171,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1583",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_08_143,LEG_08_142"
  },
  {
    "id": 1584,
    "start_hour": 173,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1584",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_08_114,LEG_08_111"
  },
  {
    "id": 1585,
    "start_hour": 173,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1585",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_08_83,LEG_08_84"
  },
  {
    "id": 1586,
    "start_hour": 176,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1586",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_08_151,LEG_08_146"
  },
  {
    "id": 1587,
    "start_hour": 181,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D1587",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_09_8,LEG_09_105,LEG_09_159,LEG_09_75"
  },
  {
    "id": 1588,
    "start_hour": 177,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1588",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_08_92,LEG_08_69"
  },
  {
    "id": 1589,
    "start_hour": 182,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1589",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_09_24"
  },
  {
    "id": 1590,
    "start_hour": 173,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1590",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_08_6,LEG_08_2"
  },
  {
    "id": 1591,
    "start_hour": 175,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1591",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_08_18,LEG_08_17"
  },
  {
    "id": 1592,
    "start_hour": 174,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1592",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_08_120,LEG_08_91"
  },
  {
    "id": 1593,
    "start_hour": 180,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1593",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_08_28"
  },
  {
    "id": 1594,
    "start_hour": 192,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1594",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_09_82,LEG_09_131"
  },
  {
    "id": 1595,
    "start_hour": 217,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1595",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_10_133,LEG_10_83,LEG_10_84"
  },
  {
    "id": 1596,
    "start_hour": 168,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1596",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_08_14,LEG_08_36"
  },
  {
    "id": 1597,
    "start_hour": 183,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1597",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_09_68,LEG_09_158"
  },
  {
    "id": 1598,
    "start_hour": 205,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1598",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_10_173,LEG_10_31"
  },
  {
    "id": 1599,
    "start_hour": 180,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1599",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_08_21"
  },
  {
    "id": 1600,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1600",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_09_144,LEG_09_114,LEG_09_111,LEG_09_112"
  },
  {
    "id": 1601,
    "start_hour": 216,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1601",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_10_101,LEG_10_155,LEG_10_156"
  },
  {
    "id": 1602,
    "start_hour": 180,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1602",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_08_153"
  },
  {
    "id": 1603,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1603",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_09_32,LEG_09_96,LEG_09_169,LEG_09_16"
  },
  {
    "id": 1604,
    "start_hour": 215,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1604",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_10_19,LEG_10_102,LEG_10_100"
  },
  {
    "id": 1605,
    "start_hour": 178,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1605",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_08_127"
  },
  {
    "id": 1606,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1606",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_09_119,LEG_09_127"
  },
  {
    "id": 1607,
    "start_hour": 218,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1607",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_10_119"
  },
  {
    "id": 1608,
    "start_hour": 228,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1608",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_11_13,LEG_11_0,LEG_11_132,LEG_11_25"
  },
  {
    "id": 1609,
    "start_hour": 178,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1609",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_08_98"
  },
  {
    "id": 1610,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1610",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_09_97,LEG_09_150"
  },
  {
    "id": 1611,
    "start_hour": 218,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1611",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_10_148,LEG_10_16"
  },
  {
    "id": 1612,
    "start_hour": 239,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1612",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_11_19,LEG_11_102,LEG_11_100"
  },
  {
    "id": 1613,
    "start_hour": 156,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1613",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_08_123,LEG_08_64,LEG_08_56"
  },
  {
    "id": 1614,
    "start_hour": 195,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1614",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_09_59,LEG_09_74,LEG_09_22,LEG_09_51"
  },
  {
    "id": 1615,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1615",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_10_144,LEG_10_88,LEG_10_126"
  },
  {
    "id": 1616,
    "start_hour": 180,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1616",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_08_190"
  },
  {
    "id": 1617,
    "start_hour": 202,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1617",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_09_192"
  },
  {
    "id": 1618,
    "start_hour": 206,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1618",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_10_117,LEG_10_115,LEG_10_76,LEG_10_134"
  },
  {
    "id": 1619,
    "start_hour": 582,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1619",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_25_44,LEG_25_52,LEG_25_192"
  },
  {
    "id": 1620,
    "start_hour": 603,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1620",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_26_155,LEG_26_83"
  },
  {
    "id": 1621,
    "start_hour": 627,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1621",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_27_69,LEG_27_33"
  },
  {
    "id": 1622,
    "start_hour": 637,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1622",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_28_164"
  },
  {
    "id": 1623,
    "start_hour": 587,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1623",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_25_10"
  },
  {
    "id": 1624,
    "start_hour": 604,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1624",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_26_12"
  },
  {
    "id": 1625,
    "start_hour": 613,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1625",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_27_44,LEG_27_10,LEG_27_150,LEG_27_145"
  },
  {
    "id": 1626,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1626",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_25_45,LEG_25_16,LEG_25_17"
  },
  {
    "id": 1627,
    "start_hour": 599,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1627",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_26_20,LEG_26_138"
  },
  {
    "id": 1628,
    "start_hour": 627,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1628",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_27_113,LEG_27_72,LEG_27_62"
  },
  {
    "id": 1629,
    "start_hour": 579,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1629",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_25_108,LEG_25_132,LEG_25_4"
  },
  {
    "id": 1630,
    "start_hour": 600,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1630",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_26_6,LEG_26_88"
  },
  {
    "id": 1631,
    "start_hour": 628,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1631",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_27_51,LEG_27_52,LEG_27_7"
  },
  {
    "id": 1632,
    "start_hour": 578,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1632",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_25_176"
  },
  {
    "id": 1633,
    "start_hour": 608,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1633",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_26_165"
  },
  {
    "id": 1634,
    "start_hour": 582,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1634",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_25_184,LEG_25_9,LEG_25_53"
  },
  {
    "id": 1635,
    "start_hour": 602,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1635",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_26_42,LEG_26_170,LEG_26_10"
  },
  {
    "id": 1636,
    "start_hour": 565,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1636",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_25_141,LEG_25_42"
  },
  {
    "id": 1637,
    "start_hour": 565,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1637",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_25_54,LEG_25_46"
  },
  {
    "id": 1638,
    "start_hour": 584,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1638",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_25_113,LEG_25_114"
  },
  {
    "id": 1639,
    "start_hour": 577,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1639",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_25_167,LEG_25_164"
  },
  {
    "id": 1640,
    "start_hour": 584,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1640",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_25_56,LEG_25_51"
  },
  {
    "id": 1641,
    "start_hour": 577,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1641",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_25_183,LEG_25_181"
  },
  {
    "id": 1642,
    "start_hour": 703,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1642",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_30_61,LEG_30_6"
  },
  {
    "id": 1643,
    "start_hour": 708,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1643",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_31_50,LEG_31_61,LEG_31_62,LEG_31_6"
  },
  {
    "id": 1644,
    "start_hour": 684,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D1644",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_30_183,LEG_30_192,LEG_30_180"
  },
  {
    "id": 1645,
    "start_hour": 721,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1645",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_31_42,LEG_31_204,LEG_31_107"
  },
  {
    "id": 1646,
    "start_hour": 698,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1646",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_30_100,LEG_30_55,LEG_30_187"
  },
  {
    "id": 1647,
    "start_hour": 723,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1647",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_31_195,LEG_31_59,LEG_31_53"
  },
  {
    "id": 1648,
    "start_hour": 700,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1648",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_30_103,LEG_30_174,LEG_30_31"
  },
  {
    "id": 1649,
    "start_hour": 726,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1649",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_31_34,LEG_31_88,LEG_31_77"
  },
  {
    "id": 1650,
    "start_hour": 700,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1650",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_30_158,LEG_30_125"
  },
  {
    "id": 1651,
    "start_hour": 709,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1651",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_31_111,LEG_31_17"
  },
  {
    "id": 1652,
    "start_hour": 708,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1652",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_31_110"
  },
  {
    "id": 1653,
    "start_hour": 698,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1653",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_30_190,LEG_30_206,LEG_30_150"
  },
  {
    "id": 1654,
    "start_hour": 721,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1654",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_31_182,LEG_31_10,LEG_31_54"
  },
  {
    "id": 1655,
    "start_hour": 686,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1655",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_30_166,LEG_30_99"
  },
  {
    "id": 1656,
    "start_hour": 712,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1656",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_31_165,LEG_31_160"
  },
  {
    "id": 1657,
    "start_hour": 684,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1657",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_30_5,LEG_30_8,LEG_30_96"
  },
  {
    "id": 1658,
    "start_hour": 708,
    "duration_hours": 2,
    "required_skill": "A320",
    "gerad_duty_id": "D1658",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_31_168"
  },
  {
    "id": 1659,
    "start_hour": 698,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1659",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_30_130,LEG_30_97"
  },
  {
    "id": 1660,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1660",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_30_185,LEG_30_175"
  },
  {
    "id": 1661,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1661",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_30_177,LEG_30_131"
  },
  {
    "id": 1662,
    "start_hour": 696,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1662",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_30_57,LEG_30_60"
  },
  {
    "id": 1663,
    "start_hour": 699,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1663",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_30_176"
  },
  {
    "id": 1664,
    "start_hour": 728,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1664",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_31_179"
  },
  {
    "id": 1665,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1665",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_30_107,LEG_30_101"
  },
  {
    "id": 1666,
    "start_hour": 686,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1666",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_30_75,LEG_30_189"
  },
  {
    "id": 1667,
    "start_hour": 709,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1667",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_31_148,LEG_31_200,LEG_31_170,LEG_31_31,LEG_31_109"
  },
  {
    "id": 1668,
    "start_hour": 703,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1668",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_30_191,LEG_30_184"
  },
  {
    "id": 1669,
    "start_hour": 708,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1669",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_31_184,LEG_31_193,LEG_31_192,LEG_31_185"
  },
  {
    "id": 1670,
    "start_hour": 702,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1670",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_30_58,LEG_30_52"
  },
  {
    "id": 1671,
    "start_hour": 705,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1671",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_30_54"
  },
  {
    "id": 1672,
    "start_hour": 709,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1672",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_31_49"
  },
  {
    "id": 1673,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1673",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_07_1,LEG_07_5"
  },
  {
    "id": 1674,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1674",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_07_154,LEG_07_155"
  },
  {
    "id": 1675,
    "start_hour": 135,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1675",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_07_3,LEG_07_23"
  },
  {
    "id": 1676,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1676",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_07_95,LEG_07_99"
  },
  {
    "id": 1677,
    "start_hour": 153,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1677",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_07_92,LEG_07_69"
  },
  {
    "id": 1678,
    "start_hour": 158,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1678",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_08_24"
  },
  {
    "id": 1679,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1679",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_07_88,LEG_07_125"
  },
  {
    "id": 1680,
    "start_hour": 158,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D1680",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_08_135,LEG_08_157"
  },
  {
    "id": 1681,
    "start_hour": 147,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1681",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_07_142,LEG_07_141"
  },
  {
    "id": 1682,
    "start_hour": 156,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1682",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_08_85,LEG_08_94,LEG_08_168,LEG_08_86"
  },
  {
    "id": 1683,
    "start_hour": 151,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1683",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_07_18,LEG_07_17"
  },
  {
    "id": 1684,
    "start_hour": 158,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1684",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_08_125,LEG_08_124"
  },
  {
    "id": 1685,
    "start_hour": 149,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1685",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_07_6,LEG_07_2"
  },
  {
    "id": 1686,
    "start_hour": 150,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1686",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_07_119,LEG_07_91"
  },
  {
    "id": 1687,
    "start_hour": 154,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1687",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_07_98"
  },
  {
    "id": 1688,
    "start_hour": 170,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1688",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_08_97"
  },
  {
    "id": 1689,
    "start_hour": 180,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1689",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_09_85,LEG_09_94,LEG_09_168,LEG_09_86"
  },
  {
    "id": 1690,
    "start_hour": 144,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1690",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_07_14,LEG_07_36"
  },
  {
    "id": 1691,
    "start_hour": 159,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1691",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_08_68,LEG_08_158"
  },
  {
    "id": 1692,
    "start_hour": 181,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1692",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_09_173,LEG_09_31"
  },
  {
    "id": 1693,
    "start_hour": 149,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1693",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_07_144,LEG_07_153,LEG_07_28"
  },
  {
    "id": 1694,
    "start_hour": 168,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1694",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_08_82,LEG_08_189"
  },
  {
    "id": 1695,
    "start_hour": 192,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1695",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_09_191"
  },
  {
    "id": 1696,
    "start_hour": 147,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1696",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_07_20,LEG_07_15,LEG_07_87"
  },
  {
    "id": 1697,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1697",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_08_89,LEG_08_16"
  },
  {
    "id": 1698,
    "start_hour": 191,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1698",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_09_19,LEG_09_102,LEG_09_100"
  },
  {
    "id": 1699,
    "start_hour": 150,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1699",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_07_115,LEG_07_77"
  },
  {
    "id": 1700,
    "start_hour": 158,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1700",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_08_117,LEG_08_115,LEG_08_76,LEG_08_134"
  },
  {
    "id": 1701,
    "start_hour": 149,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1701",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_07_83,LEG_07_84,LEG_07_140"
  },
  {
    "id": 1702,
    "start_hour": 168,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1702",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_08_122,LEG_08_121"
  },
  {
    "id": 1703,
    "start_hour": 192,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D1703",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_09_129,LEG_09_121"
  },
  {
    "id": 1704,
    "start_hour": 216,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1704",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_10_129"
  },
  {
    "id": 1705,
    "start_hour": 156,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1705",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_07_21"
  },
  {
    "id": 1706,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1706",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_08_41,LEG_08_46"
  },
  {
    "id": 1707,
    "start_hour": 192,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1707",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_09_7,LEG_09_189"
  },
  {
    "id": 1708,
    "start_hour": 216,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1708",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_10_191"
  },
  {
    "id": 1709,
    "start_hour": 326,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1709",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_15_69,LEG_15_62"
  },
  {
    "id": 1710,
    "start_hour": 326,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1710",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_15_36,LEG_15_106"
  },
  {
    "id": 1711,
    "start_hour": 338,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1711",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_15_63,LEG_15_111,LEG_15_11"
  },
  {
    "id": 1712,
    "start_hour": 359,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1712",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_16_166,LEG_16_41,LEG_16_64"
  },
  {
    "id": 1713,
    "start_hour": 338,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1713",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_15_35,LEG_15_153"
  },
  {
    "id": 1714,
    "start_hour": 348,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1714",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_16_13,LEG_16_0,LEG_16_134,LEG_16_74"
  },
  {
    "id": 1715,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1715",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_15_39,LEG_15_37"
  },
  {
    "id": 1716,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1716",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_15_67,LEG_15_68"
  },
  {
    "id": 1717,
    "start_hour": 350,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1717",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_16_69,LEG_16_66,LEG_16_39,LEG_16_37"
  },
  {
    "id": 1718,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1718",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_15_105,LEG_15_88,LEG_15_89"
  },
  {
    "id": 1719,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1719",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_16_91,LEG_16_89"
  },
  {
    "id": 1720,
    "start_hour": 386,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1720",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_17_91,LEG_17_17"
  },
  {
    "id": 1721,
    "start_hour": 407,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1721",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_18_20,LEG_18_172,LEG_18_59"
  },
  {
    "id": 1722,
    "start_hour": 342,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1722",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_15_76,LEG_15_23,LEG_15_192"
  },
  {
    "id": 1723,
    "start_hour": 363,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1723",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_16_168,LEG_16_131"
  },
  {
    "id": 1724,
    "start_hour": 386,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1724",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_17_123,LEG_17_145"
  },
  {
    "id": 1725,
    "start_hour": 408,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1725",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_18_126,LEG_18_134,LEG_18_74"
  },
  {
    "id": 1726,
    "start_hour": 111,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1726",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_06_59"
  },
  {
    "id": 1727,
    "start_hour": 148,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1727",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_07_79,LEG_07_107,LEG_07_184"
  },
  {
    "id": 1728,
    "start_hour": 168,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1728",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_08_60,LEG_08_103,LEG_08_177,LEG_08_178"
  },
  {
    "id": 1729,
    "start_hour": 193,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1729",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_09_47,LEG_09_176,LEG_09_38"
  },
  {
    "id": 1730,
    "start_hour": 126,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1730",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_06_55,LEG_06_169"
  },
  {
    "id": 1731,
    "start_hour": 134,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1731",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_07_186,LEG_07_181,LEG_07_175,LEG_07_56"
  },
  {
    "id": 1732,
    "start_hour": 171,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1732",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_08_59,LEG_08_37,LEG_08_35"
  },
  {
    "id": 1733,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1733",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_06_57,LEG_06_58"
  },
  {
    "id": 1734,
    "start_hour": 134,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1734",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_07_67,LEG_07_64,LEG_07_37,LEG_07_35"
  },
  {
    "id": 1735,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1735",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_06_91,LEG_06_150,LEG_06_76"
  },
  {
    "id": 1736,
    "start_hour": 146,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1736",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_07_89,LEG_07_27,LEG_07_58"
  },
  {
    "id": 1737,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1737",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_06_54,LEG_06_95,LEG_06_12"
  },
  {
    "id": 1738,
    "start_hour": 143,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1738",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_07_161,LEG_07_39,LEG_07_62"
  },
  {
    "id": 1739,
    "start_hour": 110,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1739",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_06_32,LEG_06_93"
  },
  {
    "id": 1740,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1740",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_14_1,LEG_14_5"
  },
  {
    "id": 1741,
    "start_hour": 312,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1741",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_14_31,LEG_14_32"
  },
  {
    "id": 1742,
    "start_hour": 315,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1742",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_14_142,LEG_14_141"
  },
  {
    "id": 1743,
    "start_hour": 317,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1743",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_14_144,LEG_14_153"
  },
  {
    "id": 1744,
    "start_hour": 312,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1744",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_14_163,LEG_14_164"
  },
  {
    "id": 1745,
    "start_hour": 317,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1745",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_14_85,LEG_14_86"
  },
  {
    "id": 1746,
    "start_hour": 314,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1746",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_14_97,LEG_14_101"
  },
  {
    "id": 1747,
    "start_hour": 321,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1747",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_14_94,LEG_14_70"
  },
  {
    "id": 1748,
    "start_hour": 326,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1748",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_15_25"
  },
  {
    "id": 1749,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1749",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_14_90,LEG_14_125"
  },
  {
    "id": 1750,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1750",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_14_119,LEG_14_93"
  },
  {
    "id": 1751,
    "start_hour": 317,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1751",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_14_6,LEG_14_2"
  },
  {
    "id": 1752,
    "start_hour": 319,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1752",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_14_18,LEG_14_17"
  },
  {
    "id": 1753,
    "start_hour": 312,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1753",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_14_14,LEG_14_37"
  },
  {
    "id": 1754,
    "start_hour": 327,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1754",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_15_70,LEG_15_162"
  },
  {
    "id": 1755,
    "start_hour": 349,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1755",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_16_177,LEG_16_32"
  },
  {
    "id": 1756,
    "start_hour": 322,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1756",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_14_126"
  },
  {
    "id": 1757,
    "start_hour": 338,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1757",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_15_123,LEG_15_193"
  },
  {
    "id": 1758,
    "start_hour": 360,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1758",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_16_195"
  },
  {
    "id": 1759,
    "start_hour": 324,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1759",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_14_4"
  },
  {
    "id": 1760,
    "start_hour": 338,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1760",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_15_143,LEG_15_81,LEG_15_112,LEG_15_48"
  },
  {
    "id": 1761,
    "start_hour": 360,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1761",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_16_7"
  },
  {
    "id": 1762,
    "start_hour": 320,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1762",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_14_150,LEG_14_145"
  },
  {
    "id": 1763,
    "start_hour": 327,
    "duration_hours": 23,
    "required_skill": "A319",
    "gerad_duty_id": "D1763",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_15_33,LEG_15_61,LEG_15_75"
  },
  {
    "id": 1764,
    "start_hour": 359,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1764",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_16_73,LEG_16_35,LEG_16_153"
  },
  {
    "id": 1765,
    "start_hour": 324,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1765",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_14_20"
  },
  {
    "id": 1766,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1766",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_15_148,LEG_15_118,LEG_15_115,LEG_15_116"
  },
  {
    "id": 1767,
    "start_hour": 360,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1767",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_16_103,LEG_16_159,LEG_16_160"
  },
  {
    "id": 1768,
    "start_hour": 324,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D1768",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_14_27"
  },
  {
    "id": 1769,
    "start_hour": 336,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1769",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_15_84,LEG_15_135"
  },
  {
    "id": 1770,
    "start_hour": 361,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1770",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_16_137,LEG_16_85,LEG_16_86"
  },
  {
    "id": 1771,
    "start_hour": 324,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1771",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_14_152"
  },
  {
    "id": 1772,
    "start_hour": 337,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1772",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_15_34,LEG_15_125"
  },
  {
    "id": 1773,
    "start_hour": 360,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1773",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_16_133,LEG_16_135"
  },
  {
    "id": 1774,
    "start_hour": 385,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1774",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_17_137,LEG_17_85,LEG_17_86"
  },
  {
    "id": 1775,
    "start_hour": 322,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1775",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_14_100"
  },
  {
    "id": 1776,
    "start_hour": 338,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1776",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_15_99,LEG_15_174"
  },
  {
    "id": 1777,
    "start_hour": 361,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1777",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_16_117,LEG_16_175"
  },
  {
    "id": 1778,
    "start_hour": 384,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1778",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_17_62,LEG_17_105,LEG_17_170"
  },
  {
    "id": 1779,
    "start_hour": 324,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1779",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_14_187"
  },
  {
    "id": 1780,
    "start_hour": 346,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1780",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_15_196"
  },
  {
    "id": 1781,
    "start_hour": 350,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1781",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_16_151,LEG_16_156,LEG_16_22"
  },
  {
    "id": 1782,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1782",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_17_148,LEG_17_90,LEG_17_130"
  },
  {
    "id": 1783,
    "start_hour": 542,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1783",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_24_69,LEG_24_62"
  },
  {
    "id": 1784,
    "start_hour": 542,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1784",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_24_36,LEG_24_106"
  },
  {
    "id": 1785,
    "start_hour": 554,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1785",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_24_63,LEG_24_111,LEG_24_11"
  },
  {
    "id": 1786,
    "start_hour": 575,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1786",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_25_166,LEG_25_41,LEG_25_64"
  },
  {
    "id": 1787,
    "start_hour": 554,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1787",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_24_35,LEG_24_153"
  },
  {
    "id": 1788,
    "start_hour": 564,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1788",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_25_13,LEG_25_0,LEG_25_85,LEG_25_86"
  },
  {
    "id": 1789,
    "start_hour": 600,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1789",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_26_17,LEG_26_35"
  },
  {
    "id": 1790,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1790",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_24_67,LEG_24_68"
  },
  {
    "id": 1791,
    "start_hour": 566,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1791",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_25_69,LEG_25_168,LEG_25_28,LEG_25_60"
  },
  {
    "id": 1792,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1792",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_24_65,LEG_24_122"
  },
  {
    "id": 1793,
    "start_hour": 566,
    "duration_hours": 19,
    "required_skill": "A321",
    "gerad_duty_id": "D1793",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_25_92,LEG_25_95"
  },
  {
    "id": 1794,
    "start_hour": 588,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1794",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_26_121,LEG_26_171,LEG_26_40"
  },
  {
    "id": 1795,
    "start_hour": 612,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D1795",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_27_139"
  },
  {
    "id": 1796,
    "start_hour": 554,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1796",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_24_105,LEG_24_59,LEG_24_58"
  },
  {
    "id": 1797,
    "start_hour": 579,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1797",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_25_61,LEG_25_58"
  },
  {
    "id": 1798,
    "start_hour": 603,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1798",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_26_59,LEG_26_36,LEG_26_34"
  },
  {
    "id": 1799,
    "start_hour": 350,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1799",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_16_36,LEG_16_106"
  },
  {
    "id": 1800,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1800",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_16_63,LEG_16_111,LEG_16_11"
  },
  {
    "id": 1801,
    "start_hour": 383,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1801",
    "gerad_crew_id": "C0190",
    "flight_ids": "LEG_17_166,LEG_17_41,LEG_17_64"
  },
  {
    "id": 1802,
    "start_hour": 362,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1802",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_16_72,LEG_16_188,LEG_16_48"
  },
  {
    "id": 1803,
    "start_hour": 384,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1803",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_17_7,LEG_17_175"
  },
  {
    "id": 1804,
    "start_hour": 408,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1804",
    "gerad_crew_id": "C0191",
    "flight_ids": "LEG_18_179,LEG_18_180,LEG_18_40"
  },
  {
    "id": 1805,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1805",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_16_67,LEG_16_68"
  },
  {
    "id": 1806,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1806",
    "gerad_crew_id": "C0192",
    "flight_ids": "LEG_17_69,LEG_17_66,LEG_17_39,LEG_17_37"
  },
  {
    "id": 1807,
    "start_hour": 366,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1807",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_16_76,LEG_16_23,LEG_16_192"
  },
  {
    "id": 1808,
    "start_hour": 387,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1808",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_17_168,LEG_17_154"
  },
  {
    "id": 1809,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1809",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_18_152,LEG_18_17"
  },
  {
    "id": 1810,
    "start_hour": 431,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1810",
    "gerad_crew_id": "C0193",
    "flight_ids": "LEG_19_20,LEG_19_164,LEG_19_55"
  },
  {
    "id": 1811,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1811",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_16_65,LEG_16_122"
  },
  {
    "id": 1812,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1812",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_17_151,LEG_17_156,LEG_17_89"
  },
  {
    "id": 1813,
    "start_hour": 410,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1813",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_18_91,LEG_18_145"
  },
  {
    "id": 1814,
    "start_hour": 432,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1814",
    "gerad_crew_id": "C0194",
    "flight_ids": "LEG_19_123,LEG_19_131,LEG_19_69"
  },
  {
    "id": 1815,
    "start_hour": 278,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1815",
    "gerad_crew_id": "C0195",
    "flight_ids": "LEG_13_35,LEG_13_98"
  },
  {
    "id": 1816,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1816",
    "gerad_crew_id": "C0196",
    "flight_ids": "LEG_13_38,LEG_13_36"
  },
  {
    "id": 1817,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1817",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_13_96,LEG_13_80,LEG_13_81"
  },
  {
    "id": 1818,
    "start_hour": 314,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1818",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_14_91,LEG_14_140"
  },
  {
    "id": 1819,
    "start_hour": 336,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1819",
    "gerad_crew_id": "C0197",
    "flight_ids": "LEG_15_126,LEG_15_134,LEG_15_74"
  },
  {
    "id": 1820,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1820",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_13_58,LEG_13_59"
  },
  {
    "id": 1821,
    "start_hour": 302,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1821",
    "gerad_crew_id": "C0198",
    "flight_ids": "LEG_14_68,LEG_14_65,LEG_14_38,LEG_14_36"
  },
  {
    "id": 1822,
    "start_hour": 290,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1822",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_13_56,LEG_13_100,LEG_13_23"
  },
  {
    "id": 1823,
    "start_hour": 312,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1823",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_14_75,LEG_14_169"
  },
  {
    "id": 1824,
    "start_hour": 336,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1824",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_15_179,LEG_15_144,LEG_15_57,LEG_15_189"
  },
  {
    "id": 1825,
    "start_hour": 360,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1825",
    "gerad_crew_id": "C0199",
    "flight_ids": "LEG_16_179,LEG_16_180,LEG_16_40"
  },
  {
    "id": 1826,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1826",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_16_1,LEG_16_5"
  },
  {
    "id": 1827,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1827",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_16_97,LEG_16_101"
  },
  {
    "id": 1828,
    "start_hour": 351,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1828",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_16_3,LEG_16_24"
  },
  {
    "id": 1829,
    "start_hour": 363,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1829",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_16_147,LEG_16_146"
  },
  {
    "id": 1830,
    "start_hour": 372,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1830",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_17_87,LEG_17_96,LEG_17_172,LEG_17_88"
  },
  {
    "id": 1831,
    "start_hour": 364,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1831",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_16_136,LEG_16_26"
  },
  {
    "id": 1832,
    "start_hour": 372,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1832",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_17_13,LEG_17_0,LEG_17_31,LEG_17_30"
  },
  {
    "id": 1833,
    "start_hour": 366,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1833",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_16_120,LEG_16_79"
  },
  {
    "id": 1834,
    "start_hour": 373,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D1834",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_17_8,LEG_17_107,LEG_17_163,LEG_17_77"
  },
  {
    "id": 1835,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1835",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_16_90,LEG_16_130"
  },
  {
    "id": 1836,
    "start_hour": 374,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1836",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_17_139,LEG_17_161"
  },
  {
    "id": 1837,
    "start_hour": 367,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1837",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_16_19,LEG_16_18"
  },
  {
    "id": 1838,
    "start_hour": 374,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1838",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_17_129,LEG_17_128"
  },
  {
    "id": 1839,
    "start_hour": 367,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1839",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_16_78,LEG_16_138"
  },
  {
    "id": 1840,
    "start_hour": 375,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1840",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_17_33,LEG_17_109,LEG_17_110,LEG_17_14"
  },
  {
    "id": 1841,
    "start_hour": 369,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1841",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_16_94,LEG_16_71"
  },
  {
    "id": 1842,
    "start_hour": 374,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1842",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_17_25"
  },
  {
    "id": 1843,
    "start_hour": 365,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1843",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_16_6,LEG_16_2"
  },
  {
    "id": 1844,
    "start_hour": 366,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1844",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_16_124,LEG_16_93"
  },
  {
    "id": 1845,
    "start_hour": 370,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D1845",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_16_100"
  },
  {
    "id": 1846,
    "start_hour": 386,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1846",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_17_99,LEG_17_27"
  },
  {
    "id": 1847,
    "start_hour": 408,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1847",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_18_171,LEG_18_31,LEG_18_30"
  },
  {
    "id": 1848,
    "start_hour": 365,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1848",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_16_149,LEG_16_158,LEG_16_29"
  },
  {
    "id": 1849,
    "start_hour": 384,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1849",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_17_84,LEG_17_135"
  },
  {
    "id": 1850,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1850",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_18_137,LEG_18_85,LEG_18_86"
  },
  {
    "id": 1851,
    "start_hour": 360,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D1851",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_16_15,LEG_16_38"
  },
  {
    "id": 1852,
    "start_hour": 375,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1852",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_17_70,LEG_17_162"
  },
  {
    "id": 1853,
    "start_hour": 397,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1853",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_18_177,LEG_18_32"
  },
  {
    "id": 1854,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1854",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_16_155,LEG_16_150"
  },
  {
    "id": 1855,
    "start_hour": 374,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D1855",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_17_92,LEG_17_95"
  },
  {
    "id": 1856,
    "start_hour": 397,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D1856",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_18_8,LEG_18_107,LEG_18_163,LEG_18_77"
  },
  {
    "id": 1857,
    "start_hour": 363,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1857",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_16_31,LEG_16_30,LEG_16_27"
  },
  {
    "id": 1858,
    "start_hour": 384,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1858",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_17_171,LEG_17_21,LEG_17_47,LEG_17_182"
  },
  {
    "id": 1859,
    "start_hour": 409,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1859",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_18_49,LEG_18_81,LEG_18_112,LEG_18_189"
  },
  {
    "id": 1860,
    "start_hour": 432,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1860",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_19_58,LEG_19_98,LEG_19_162"
  },
  {
    "id": 1861,
    "start_hour": 608,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1861",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_26_55,LEG_26_25"
  },
  {
    "id": 1862,
    "start_hour": 625,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1862",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_27_1,LEG_27_116"
  },
  {
    "id": 1863,
    "start_hour": 651,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1863",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_28_140,LEG_28_87,LEG_28_76"
  },
  {
    "id": 1864,
    "start_hour": 604,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1864",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_26_109,LEG_26_30,LEG_26_147"
  },
  {
    "id": 1865,
    "start_hour": 626,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1865",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_27_154,LEG_27_137"
  },
  {
    "id": 1866,
    "start_hour": 651,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1866",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_28_189,LEG_28_188,LEG_28_181"
  },
  {
    "id": 1867,
    "start_hour": 601,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1867",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_26_169,LEG_26_168,LEG_26_47"
  },
  {
    "id": 1868,
    "start_hour": 626,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1868",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_27_64,LEG_27_28,LEG_27_84"
  },
  {
    "id": 1869,
    "start_hour": 601,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1869",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_26_44,LEG_26_46"
  },
  {
    "id": 1870,
    "start_hour": 601,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1870",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_26_154,LEG_26_152"
  },
  {
    "id": 1871,
    "start_hour": 599,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1871",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_26_99,LEG_26_108,LEG_26_15,LEG_26_106"
  },
  {
    "id": 1872,
    "start_hour": 589,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1872",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_26_51,LEG_26_45"
  },
  {
    "id": 1873,
    "start_hour": 589,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1873",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_26_134,LEG_26_38"
  },
  {
    "id": 1874,
    "start_hour": 398,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1874",
    "gerad_crew_id": "C0200",
    "flight_ids": "LEG_18_69,LEG_18_62"
  },
  {
    "id": 1875,
    "start_hour": 398,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1875",
    "gerad_crew_id": "C0201",
    "flight_ids": "LEG_18_36,LEG_18_106"
  },
  {
    "id": 1876,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1876",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_18_63,LEG_18_111,LEG_18_182"
  },
  {
    "id": 1877,
    "start_hour": 433,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1877",
    "gerad_crew_id": "C0202",
    "flight_ids": "LEG_19_46,LEG_19_172,LEG_19_36"
  },
  {
    "id": 1878,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1878",
    "gerad_crew_id": "C0203",
    "flight_ids": "LEG_18_39,LEG_18_37"
  },
  {
    "id": 1879,
    "start_hour": 433,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1879",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_19_94,LEG_19_133,LEG_19_105"
  },
  {
    "id": 1880,
    "start_hour": 455,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1880",
    "gerad_crew_id": "C0204",
    "flight_ids": "LEG_20_13"
  },
  {
    "id": 1881,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1881",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_18_67,LEG_18_68"
  },
  {
    "id": 1882,
    "start_hour": 422,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1882",
    "gerad_crew_id": "C0205",
    "flight_ids": "LEG_19_64,LEG_19_61,LEG_19_62,LEG_19_63"
  },
  {
    "id": 1883,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1883",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_18_65,LEG_18_122"
  },
  {
    "id": 1884,
    "start_hour": 422,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1884",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_19_86,LEG_19_89,LEG_19_185"
  },
  {
    "id": 1885,
    "start_hour": 466,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1885",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_20_181"
  },
  {
    "id": 1886,
    "start_hour": 469,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1886",
    "gerad_crew_id": "C0206",
    "flight_ids": "LEG_21_8,LEG_21_107,LEG_21_180,LEG_21_40"
  },
  {
    "id": 1887,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1887",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_18_105,LEG_18_88,LEG_18_89"
  },
  {
    "id": 1888,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1888",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_19_85,LEG_19_148"
  },
  {
    "id": 1889,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1889",
    "gerad_crew_id": "C0207",
    "flight_ids": "LEG_20_142,LEG_20_25,LEG_20_54"
  },
  {
    "id": 1890,
    "start_hour": 410,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1890",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_18_72,LEG_18_188,LEG_18_48"
  },
  {
    "id": 1891,
    "start_hour": 432,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1891",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_19_6,LEG_19_167"
  },
  {
    "id": 1892,
    "start_hour": 457,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1892",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_20_108,LEG_20_91,LEG_20_160,LEG_20_144"
  },
  {
    "id": 1893,
    "start_hour": 482,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1893",
    "gerad_crew_id": "C0208",
    "flight_ids": "LEG_21_152,LEG_21_28,LEG_21_60"
  },
  {
    "id": 1894,
    "start_hour": 480,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1894",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_21_169,LEG_21_170"
  },
  {
    "id": 1895,
    "start_hour": 471,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1895",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_21_3,LEG_21_24"
  },
  {
    "id": 1896,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1896",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_21_1,LEG_21_5"
  },
  {
    "id": 1897,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D1897",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_21_97,LEG_21_101"
  },
  {
    "id": 1898,
    "start_hour": 489,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1898",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_21_94,LEG_21_71"
  },
  {
    "id": 1899,
    "start_hour": 494,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1899",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_22_25"
  },
  {
    "id": 1900,
    "start_hour": 487,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1900",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_21_19,LEG_21_18"
  },
  {
    "id": 1901,
    "start_hour": 493,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1901",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_22_8,LEG_22_107,LEG_22_162,LEG_22_77"
  },
  {
    "id": 1902,
    "start_hour": 480,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D1902",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_21_15,LEG_21_38,LEG_21_75"
  },
  {
    "id": 1903,
    "start_hour": 503,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1903",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_22_73,LEG_22_35,LEG_22_153"
  },
  {
    "id": 1904,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1904",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_21_31,LEG_21_30,LEG_21_27"
  },
  {
    "id": 1905,
    "start_hour": 504,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1905",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_22_170,LEG_22_31,LEG_22_30"
  },
  {
    "id": 1906,
    "start_hour": 485,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1906",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_21_6,LEG_21_2"
  },
  {
    "id": 1907,
    "start_hour": 470,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1907",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_21_121,LEG_21_119,LEG_21_78,LEG_21_138"
  },
  {
    "id": 1908,
    "start_hour": 470,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1908",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_21_129,LEG_21_128,LEG_21_157"
  },
  {
    "id": 1909,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1909",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_22_34,LEG_22_17"
  },
  {
    "id": 1910,
    "start_hour": 527,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1910",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_23_20,LEG_23_104,LEG_23_102"
  },
  {
    "id": 1911,
    "start_hour": 468,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D1911",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_21_127,LEG_21_66,LEG_21_76,LEG_21_23,LEG_21_192"
  },
  {
    "id": 1912,
    "start_hour": 507,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1912",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_22_167,LEG_22_155,LEG_22_150"
  },
  {
    "id": 1913,
    "start_hour": 517,
    "duration_hours": 20,
    "required_skill": "A319",
    "gerad_duty_id": "D1913",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_23_8,LEG_23_107,LEG_23_163,LEG_23_77"
  },
  {
    "id": 1914,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1914",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_21_124,LEG_21_93"
  },
  {
    "id": 1915,
    "start_hour": 495,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1915",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_22_33,LEG_22_109,LEG_22_181"
  },
  {
    "id": 1916,
    "start_hour": 529,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1916",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_23_49,LEG_23_110,LEG_23_14"
  },
  {
    "id": 1917,
    "start_hour": 470,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1917",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_21_151,LEG_21_156,LEG_21_22"
  },
  {
    "id": 1918,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1918",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_22_148,LEG_22_90,LEG_22_130"
  },
  {
    "id": 1919,
    "start_hour": 488,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1919",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_21_155,LEG_21_150"
  },
  {
    "id": 1920,
    "start_hour": 494,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1920",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_22_92,LEG_22_95,LEG_22_154"
  },
  {
    "id": 1921,
    "start_hour": 530,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1921",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_23_152"
  },
  {
    "id": 1922,
    "start_hour": 540,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D1922",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_24_13,LEG_24_0,LEG_24_172,LEG_24_88"
  },
  {
    "id": 1923,
    "start_hour": 484,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1923",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_21_136,LEG_21_26"
  },
  {
    "id": 1924,
    "start_hour": 492,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D1924",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_22_13,LEG_22_0,LEG_22_104,LEG_22_102,LEG_22_192"
  },
  {
    "id": 1925,
    "start_hour": 528,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1925",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_23_195,LEG_23_135"
  },
  {
    "id": 1926,
    "start_hour": 553,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1926",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_24_137,LEG_24_85,LEG_24_86"
  },
  {
    "id": 1927,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1927",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_21_90,LEG_21_130"
  },
  {
    "id": 1928,
    "start_hour": 494,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D1928",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_22_139,LEG_22_161"
  },
  {
    "id": 1929,
    "start_hour": 516,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1929",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_23_127,LEG_23_117,LEG_23_98,LEG_23_173,LEG_23_116"
  },
  {
    "id": 1930,
    "start_hour": 552,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1930",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_24_103,LEG_24_159,LEG_24_160"
  },
  {
    "id": 1931,
    "start_hour": 485,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1931",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_21_149,LEG_21_158,LEG_21_29"
  },
  {
    "id": 1932,
    "start_hour": 504,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1932",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_22_84,LEG_22_125"
  },
  {
    "id": 1933,
    "start_hour": 528,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1933",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_23_133,LEG_23_125"
  },
  {
    "id": 1934,
    "start_hour": 552,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1934",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_24_133"
  },
  {
    "id": 1935,
    "start_hour": 490,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D1935",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_21_100"
  },
  {
    "id": 1936,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1936",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_22_99,LEG_22_89"
  },
  {
    "id": 1937,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1937",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_23_91,LEG_23_154"
  },
  {
    "id": 1938,
    "start_hour": 554,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1938",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_24_152,LEG_24_120,LEG_24_79"
  },
  {
    "id": 1939,
    "start_hour": 408,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1939",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_18_169,LEG_18_170"
  },
  {
    "id": 1940,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1940",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_18_1,LEG_18_5"
  },
  {
    "id": 1941,
    "start_hour": 399,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D1941",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_18_3,LEG_18_24"
  },
  {
    "id": 1942,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1942",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_18_97,LEG_18_101"
  },
  {
    "id": 1943,
    "start_hour": 412,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1943",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_18_136,LEG_18_26"
  },
  {
    "id": 1944,
    "start_hour": 420,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D1944",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_19_14,LEG_19_0,LEG_19_95,LEG_19_97"
  },
  {
    "id": 1945,
    "start_hour": 417,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1945",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_18_94,LEG_18_71"
  },
  {
    "id": 1946,
    "start_hour": 422,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D1946",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_19_22"
  },
  {
    "id": 1947,
    "start_hour": 411,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1947",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_18_147,LEG_18_146"
  },
  {
    "id": 1948,
    "start_hour": 420,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D1948",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_19_81,LEG_19_90,LEG_19_141,LEG_19_140"
  },
  {
    "id": 1949,
    "start_hour": 408,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D1949",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_18_15,LEG_18_38,LEG_18_75"
  },
  {
    "id": 1950,
    "start_hour": 431,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1950",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_19_68,LEG_19_31,LEG_19_147"
  },
  {
    "id": 1951,
    "start_hour": 398,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1951",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_18_129,LEG_18_128,LEG_18_157"
  },
  {
    "id": 1952,
    "start_hour": 433,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1952",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_19_30,LEG_19_91,LEG_19_165"
  },
  {
    "id": 1953,
    "start_hour": 398,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D1953",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_18_121,LEG_18_119,LEG_18_78,LEG_18_138"
  },
  {
    "id": 1954,
    "start_hour": 413,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1954",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_18_6,LEG_18_2"
  },
  {
    "id": 1955,
    "start_hour": 416,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1955",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_18_28,LEG_18_60"
  },
  {
    "id": 1956,
    "start_hour": 423,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D1956",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_19_66"
  },
  {
    "id": 1957,
    "start_hour": 457,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1957",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_20_95,LEG_20_139,LEG_20_148"
  },
  {
    "id": 1958,
    "start_hour": 396,
    "duration_hours": 27,
    "required_skill": "A319",
    "gerad_duty_id": "D1958",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_18_127,LEG_18_66,LEG_18_76,LEG_18_23,LEG_18_53"
  },
  {
    "id": 1959,
    "start_hour": 434,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1959",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_19_40,LEG_19_15,LEG_19_107,LEG_19_99"
  },
  {
    "id": 1960,
    "start_hour": 456,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1960",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_20_98,LEG_20_152,LEG_20_68"
  },
  {
    "id": 1961,
    "start_hour": 398,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D1961",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_18_92,LEG_18_95"
  },
  {
    "id": 1962,
    "start_hour": 420,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1962",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_19_124,LEG_19_114,LEG_19_117,LEG_19_74"
  },
  {
    "id": 1963,
    "start_hour": 414,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D1963",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_18_120,LEG_18_79"
  },
  {
    "id": 1964,
    "start_hour": 422,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D1964",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_19_118,LEG_19_116,LEG_19_73,LEG_19_135"
  },
  {
    "id": 1965,
    "start_hour": 413,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D1965",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_18_149,LEG_18_158,LEG_18_29"
  },
  {
    "id": 1966,
    "start_hour": 432,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D1966",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_19_78,LEG_19_122"
  },
  {
    "id": 1967,
    "start_hour": 456,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D1967",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_20_124,LEG_20_116"
  },
  {
    "id": 1968,
    "start_hour": 480,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D1968",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_21_133,LEG_21_98,LEG_21_173"
  },
  {
    "id": 1969,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1969",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_18_124,LEG_18_93"
  },
  {
    "id": 1970,
    "start_hour": 423,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D1970",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_19_28,LEG_19_106,LEG_19_53,LEG_19_139"
  },
  {
    "id": 1971,
    "start_hour": 456,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1971",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_20_117,LEG_20_126"
  },
  {
    "id": 1972,
    "start_hour": 481,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D1972",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_21_137,LEG_21_85,LEG_21_86"
  },
  {
    "id": 1973,
    "start_hour": 415,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1973",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_18_19,LEG_18_18"
  },
  {
    "id": 1974,
    "start_hour": 422,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D1974",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_19_126,LEG_19_125,LEG_19_151"
  },
  {
    "id": 1975,
    "start_hour": 457,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D1975",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_20_31,LEG_20_178"
  },
  {
    "id": 1976,
    "start_hour": 480,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D1976",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_21_195"
  },
  {
    "id": 1977,
    "start_hour": 414,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1977",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_18_90,LEG_18_130"
  },
  {
    "id": 1978,
    "start_hour": 422,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D1978",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_19_136,LEG_19_156"
  },
  {
    "id": 1979,
    "start_hour": 444,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D1979",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_20_118,LEG_20_158,LEG_20_15"
  },
  {
    "id": 1980,
    "start_hour": 479,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D1980",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_21_20,LEG_21_104,LEG_21_102"
  },
  {
    "id": 1981,
    "start_hour": 398,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D1981",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_18_151,LEG_18_156,LEG_18_194"
  },
  {
    "id": 1982,
    "start_hour": 442,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D1982",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_19_187"
  },
  {
    "id": 1983,
    "start_hour": 446,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1983",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_20_112,LEG_20_110,LEG_20_69,LEG_20_129"
  },
  {
    "id": 1984,
    "start_hour": 418,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D1984",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_18_100"
  },
  {
    "id": 1985,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1985",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_19_92,LEG_19_83"
  },
  {
    "id": 1986,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1986",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_20_84,LEG_20_161"
  },
  {
    "id": 1987,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D1987",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_21_117,LEG_21_120,LEG_21_79"
  },
  {
    "id": 1988,
    "start_hour": 229,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1988",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_11_52,LEG_11_44"
  },
  {
    "id": 1989,
    "start_hour": 242,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D1989",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_11_172"
  },
  {
    "id": 1990,
    "start_hour": 241,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1990",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_11_163,LEG_11_160"
  },
  {
    "id": 1991,
    "start_hour": 229,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D1991",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_11_137,LEG_11_40"
  },
  {
    "id": 1992,
    "start_hour": 243,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D1992",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_11_106,LEG_11_128"
  },
  {
    "id": 1993,
    "start_hour": 253,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D1993",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_12_8,LEG_12_98,LEG_12_171,LEG_12_173"
  },
  {
    "id": 1994,
    "start_hour": 241,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D1994",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_11_43,LEG_11_45,LEG_11_11"
  },
  {
    "id": 1995,
    "start_hour": 263,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D1995",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_12_150,LEG_12_180,LEG_12_15"
  },
  {
    "id": 1996,
    "start_hour": 248,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D1996",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_11_54,LEG_11_49"
  },
  {
    "id": 1997,
    "start_hour": 248,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D1997",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_11_109,LEG_11_110"
  },
  {
    "id": 1998,
    "start_hour": 251,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D1998",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_11_10"
  },
  {
    "id": 1999,
    "start_hour": 268,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D1999",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_12_12,LEG_12_175"
  },
  {
    "id": 2000,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2000",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_13_169,LEG_13_168,LEG_13_8"
  },
  {
    "id": 2001,
    "start_hour": 241,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2001",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_11_179,LEG_11_57,LEG_11_56"
  },
  {
    "id": 2002,
    "start_hour": 267,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2002",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_12_55,LEG_12_52"
  },
  {
    "id": 2003,
    "start_hour": 291,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2003",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_13_55,LEG_13_67,LEG_13_20"
  },
  {
    "id": 2004,
    "start_hour": 249,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2004",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_11_136,LEG_11_138"
  },
  {
    "id": 2005,
    "start_hour": 253,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2005",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_12_74"
  },
  {
    "id": 2006,
    "start_hour": 288,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2006",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_13_72,LEG_13_132,LEG_13_51,LEG_13_44"
  },
  {
    "id": 2007,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2007",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_14_138,LEG_14_181,LEG_14_184"
  },
  {
    "id": 2008,
    "start_hour": 251,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2008",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_11_188"
  },
  {
    "id": 2009,
    "start_hour": 267,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2009",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_12_152,LEG_12_85"
  },
  {
    "id": 2010,
    "start_hour": 292,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2010",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_13_37,LEG_13_13"
  },
  {
    "id": 2011,
    "start_hour": 397,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2011",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_18_141,LEG_18_42"
  },
  {
    "id": 2012,
    "start_hour": 409,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2012",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_18_167,LEG_18_164"
  },
  {
    "id": 2013,
    "start_hour": 397,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2013",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_18_54,LEG_18_46"
  },
  {
    "id": 2014,
    "start_hour": 416,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2014",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_18_178"
  },
  {
    "id": 2015,
    "start_hour": 411,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2015",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_18_108,LEG_18_132,LEG_18_4"
  },
  {
    "id": 2016,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2016",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_19_138,LEG_19_179,LEG_19_181"
  },
  {
    "id": 2017,
    "start_hour": 419,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2017",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_18_10"
  },
  {
    "id": 2018,
    "start_hour": 436,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2018",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_19_12"
  },
  {
    "id": 2019,
    "start_hour": 412,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2019",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_18_187,LEG_18_190"
  },
  {
    "id": 2020,
    "start_hour": 422,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2020",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_19_182,LEG_19_178,LEG_19_47,LEG_19_50"
  },
  {
    "id": 2021,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2021",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_18_183,LEG_18_181,LEG_18_165,LEG_18_82"
  },
  {
    "id": 2022,
    "start_hour": 416,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2022",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_18_56,LEG_18_51"
  },
  {
    "id": 2023,
    "start_hour": 416,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2023",
    "gerad_crew_id": "C0171",
    "flight_ids": "LEG_18_113,LEG_18_114"
  },
  {
    "id": 2024,
    "start_hour": 417,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2024",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_18_140,LEG_18_142"
  },
  {
    "id": 2025,
    "start_hour": 421,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2025",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_19_77"
  },
  {
    "id": 2026,
    "start_hour": 456,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2026",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_20_71,LEG_20_135,LEG_20_52,LEG_20_169"
  },
  {
    "id": 2027,
    "start_hour": 481,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2027",
    "gerad_crew_id": "C0172",
    "flight_ids": "LEG_21_49,LEG_21_187,LEG_21_190"
  },
  {
    "id": 2028,
    "start_hour": 409,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2028",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_18_45,LEG_18_16,LEG_18_174"
  },
  {
    "id": 2029,
    "start_hour": 435,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2029",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_19_160,LEG_19_128"
  },
  {
    "id": 2030,
    "start_hour": 458,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2030",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_20_114,LEG_20_162"
  },
  {
    "id": 2031,
    "start_hour": 480,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2031",
    "gerad_crew_id": "C0173",
    "flight_ids": "LEG_21_62,LEG_21_72,LEG_21_188"
  },
  {
    "id": 2032,
    "start_hour": 410,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2032",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_18_176"
  },
  {
    "id": 2033,
    "start_hour": 440,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2033",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_19_170"
  },
  {
    "id": 2034,
    "start_hour": 445,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2034",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_20_74"
  },
  {
    "id": 2035,
    "start_hour": 480,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2035",
    "gerad_crew_id": "C0174",
    "flight_ids": "LEG_21_80,LEG_21_44,LEG_21_52"
  },
  {
    "id": 2036,
    "start_hour": 385,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2036",
    "gerad_crew_id": "C0175",
    "flight_ids": "LEG_17_167,LEG_17_164"
  },
  {
    "id": 2037,
    "start_hour": 373,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2037",
    "gerad_crew_id": "C0176",
    "flight_ids": "LEG_17_141,LEG_17_42"
  },
  {
    "id": 2038,
    "start_hour": 373,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2038",
    "gerad_crew_id": "C0177",
    "flight_ids": "LEG_17_54,LEG_17_46"
  },
  {
    "id": 2039,
    "start_hour": 386,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2039",
    "gerad_crew_id": "C0178",
    "flight_ids": "LEG_17_176"
  },
  {
    "id": 2040,
    "start_hour": 387,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2040",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_17_108,LEG_17_132"
  },
  {
    "id": 2041,
    "start_hour": 396,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D2041",
    "gerad_crew_id": "C0179",
    "flight_ids": "LEG_18_87,LEG_18_96,LEG_18_21,LEG_18_47"
  },
  {
    "id": 2042,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2042",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_17_183,LEG_17_59,LEG_17_58"
  },
  {
    "id": 2043,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2043",
    "gerad_crew_id": "C0180",
    "flight_ids": "LEG_18_109,LEG_18_44,LEG_18_52"
  },
  {
    "id": 2044,
    "start_hour": 395,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2044",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_17_10"
  },
  {
    "id": 2045,
    "start_hour": 412,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2045",
    "gerad_crew_id": "C0181",
    "flight_ids": "LEG_18_12"
  },
  {
    "id": 2046,
    "start_hour": 392,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2046",
    "gerad_crew_id": "C0182",
    "flight_ids": "LEG_17_56,LEG_17_51"
  },
  {
    "id": 2047,
    "start_hour": 392,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2047",
    "gerad_crew_id": "C0183",
    "flight_ids": "LEG_17_113,LEG_17_114"
  },
  {
    "id": 2048,
    "start_hour": 393,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2048",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_17_140,LEG_17_142"
  },
  {
    "id": 2049,
    "start_hour": 397,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2049",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_18_83"
  },
  {
    "id": 2050,
    "start_hour": 432,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2050",
    "gerad_crew_id": "C0118",
    "flight_ids": "LEG_19_75,LEG_19_41,LEG_19_48"
  },
  {
    "id": 2051,
    "start_hour": 395,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2051",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_17_192"
  },
  {
    "id": 2052,
    "start_hour": 411,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2052",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_18_168,LEG_18_155,LEG_18_150"
  },
  {
    "id": 2053,
    "start_hour": 421,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2053",
    "gerad_crew_id": "C0119",
    "flight_ids": "LEG_19_8,LEG_19_102,LEG_19_103,LEG_19_129,LEG_19_153"
  },
  {
    "id": 2054,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2054",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_17_45,LEG_17_16,LEG_17_174"
  },
  {
    "id": 2055,
    "start_hour": 409,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2055",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_18_117,LEG_18_98,LEG_18_173,LEG_18_154"
  },
  {
    "id": 2056,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2056",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_19_146,LEG_19_166"
  },
  {
    "id": 2057,
    "start_hour": 458,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2057",
    "gerad_crew_id": "C0120",
    "flight_ids": "LEG_20_172,LEG_20_40,LEG_20_47"
  },
  {
    "id": 2058,
    "start_hour": 337,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2058",
    "gerad_crew_id": "C0121",
    "flight_ids": "LEG_15_183,LEG_15_181"
  },
  {
    "id": 2059,
    "start_hour": 325,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2059",
    "gerad_crew_id": "C0122",
    "flight_ids": "LEG_15_54,LEG_15_46"
  },
  {
    "id": 2060,
    "start_hour": 337,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2060",
    "gerad_crew_id": "C0123",
    "flight_ids": "LEG_15_167,LEG_15_164"
  },
  {
    "id": 2061,
    "start_hour": 325,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2061",
    "gerad_crew_id": "C0124",
    "flight_ids": "LEG_15_141,LEG_15_42"
  },
  {
    "id": 2062,
    "start_hour": 344,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2062",
    "gerad_crew_id": "C0125",
    "flight_ids": "LEG_15_178"
  },
  {
    "id": 2063,
    "start_hour": 341,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2063",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_15_50,LEG_15_55"
  },
  {
    "id": 2064,
    "start_hour": 350,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2064",
    "gerad_crew_id": "C0126",
    "flight_ids": "LEG_16_191,LEG_16_186,LEG_16_50,LEG_16_55"
  },
  {
    "id": 2065,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2065",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_15_41,LEG_15_64,LEG_15_58"
  },
  {
    "id": 2066,
    "start_hour": 362,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2066",
    "gerad_crew_id": "C0127",
    "flight_ids": "LEG_16_109,LEG_16_184,LEG_16_9"
  },
  {
    "id": 2067,
    "start_hour": 344,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2067",
    "gerad_crew_id": "C0128",
    "flight_ids": "LEG_15_113,LEG_15_114"
  },
  {
    "id": 2068,
    "start_hour": 344,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2068",
    "gerad_crew_id": "C0129",
    "flight_ids": "LEG_15_56,LEG_15_51"
  },
  {
    "id": 2069,
    "start_hour": 347,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2069",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_15_10"
  },
  {
    "id": 2070,
    "start_hour": 364,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2070",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_16_12,LEG_16_53"
  },
  {
    "id": 2071,
    "start_hour": 386,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2071",
    "gerad_crew_id": "C0130",
    "flight_ids": "LEG_17_43,LEG_17_184,LEG_17_9"
  },
  {
    "id": 2072,
    "start_hour": 338,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2072",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_15_176"
  },
  {
    "id": 2073,
    "start_hour": 368,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2073",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_16_178"
  },
  {
    "id": 2074,
    "start_hour": 374,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2074",
    "gerad_crew_id": "C0131",
    "flight_ids": "LEG_17_191,LEG_17_186,LEG_17_50,LEG_17_55"
  },
  {
    "id": 2075,
    "start_hour": 339,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2075",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_15_108,LEG_15_132,LEG_15_22"
  },
  {
    "id": 2076,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2076",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_16_148,LEG_16_174"
  },
  {
    "id": 2077,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2077",
    "gerad_crew_id": "C0132",
    "flight_ids": "LEG_17_185,LEG_17_44,LEG_17_52"
  },
  {
    "id": 2078,
    "start_hour": 345,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2078",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_15_165,LEG_15_82"
  },
  {
    "id": 2079,
    "start_hour": 349,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2079",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_16_83"
  },
  {
    "id": 2080,
    "start_hour": 384,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2080",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_17_80,LEG_17_144,LEG_17_57,LEG_17_189"
  },
  {
    "id": 2081,
    "start_hour": 410,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2081",
    "gerad_crew_id": "C0133",
    "flight_ids": "LEG_18_185,LEG_18_184,LEG_18_9"
  },
  {
    "id": 2082,
    "start_hour": 2,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2082",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_01_60,LEG_01_163,LEG_01_39"
  },
  {
    "id": 2083,
    "start_hour": 24,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2083",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_02_7,LEG_02_170"
  },
  {
    "id": 2084,
    "start_hour": 51,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2084",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_03_164,LEG_03_141"
  },
  {
    "id": 2085,
    "start_hour": 72,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2085",
    "gerad_crew_id": "C0209",
    "flight_ids": "LEG_04_122,LEG_04_130,LEG_04_72"
  },
  {
    "id": 2086,
    "start_hour": 6,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2086",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_01_54,LEG_01_165"
  },
  {
    "id": 2087,
    "start_hour": 14,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2087",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_02_186,LEG_02_181,LEG_02_78,LEG_02_107,LEG_02_184"
  },
  {
    "id": 2088,
    "start_hour": 48,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2088",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_03_175,LEG_03_140,LEG_03_55,LEG_03_185"
  },
  {
    "id": 2089,
    "start_hour": 72,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2089",
    "gerad_crew_id": "C0210",
    "flight_ids": "LEG_04_175,LEG_04_176,LEG_04_38"
  },
  {
    "id": 2090,
    "start_hour": 6,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2090",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_01_64,LEG_01_19"
  },
  {
    "id": 2091,
    "start_hour": 13,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2091",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_02_80"
  },
  {
    "id": 2092,
    "start_hour": 48,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2092",
    "gerad_crew_id": "C0211",
    "flight_ids": "LEG_03_78,LEG_03_176,LEG_03_38"
  },
  {
    "id": 2093,
    "start_hour": 2,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2093",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_01_87,LEG_01_146,LEG_01_129"
  },
  {
    "id": 2094,
    "start_hour": 26,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2094",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_02_147,LEG_02_140"
  },
  {
    "id": 2095,
    "start_hour": 48,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2095",
    "gerad_crew_id": "C0212",
    "flight_ids": "LEG_03_122,LEG_03_130,LEG_03_72"
  },
  {
    "id": 2096,
    "start_hour": 2,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2096",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_01_29,LEG_01_128"
  },
  {
    "id": 2097,
    "start_hour": 26,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2097",
    "gerad_crew_id": "C0213",
    "flight_ids": "LEG_02_118,LEG_02_27,LEG_02_58"
  },
  {
    "id": 2098,
    "start_hour": -1,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2098",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_01_20,LEG_01_112"
  },
  {
    "id": 2099,
    "start_hour": 25,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2099",
    "gerad_crew_id": "C0214",
    "flight_ids": "LEG_02_132,LEG_02_129,LEG_02_72"
  },
  {
    "id": 2100,
    "start_hour": 2,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2100",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_01_53,LEG_01_91,LEG_01_9"
  },
  {
    "id": 2101,
    "start_hour": 23,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2101",
    "gerad_crew_id": "C0215",
    "flight_ids": "LEG_02_161,LEG_02_39,LEG_02_62"
  },
  {
    "id": 2102,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2102",
    "gerad_crew_id": "C0216",
    "flight_ids": "LEG_01_56,LEG_01_57"
  },
  {
    "id": 2103,
    "start_hour": 6,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2103",
    "gerad_crew_id": "C0217",
    "flight_ids": "LEG_01_32,LEG_01_30"
  },
  {
    "id": 2104,
    "start_hour": 4,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2104",
    "gerad_crew_id": "C0218",
    "flight_ids": "LEG_01_111,LEG_01_62"
  },
  {
    "id": 2105,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2105",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_10_1,LEG_10_5"
  },
  {
    "id": 2106,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2106",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_10_95,LEG_10_99"
  },
  {
    "id": 2107,
    "start_hour": 207,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2107",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_10_3,LEG_10_23"
  },
  {
    "id": 2108,
    "start_hour": 222,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2108",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_10_120,LEG_10_91"
  },
  {
    "id": 2109,
    "start_hour": 230,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2109",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_11_135,LEG_11_157"
  },
  {
    "id": 2110,
    "start_hour": 223,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2110",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_10_18,LEG_10_17"
  },
  {
    "id": 2111,
    "start_hour": 230,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2111",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_11_125,LEG_11_124"
  },
  {
    "id": 2112,
    "start_hour": 225,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2112",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_10_92,LEG_10_69"
  },
  {
    "id": 2113,
    "start_hour": 230,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2113",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_11_24"
  },
  {
    "id": 2114,
    "start_hour": 219,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2114",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_10_143,LEG_10_142"
  },
  {
    "id": 2115,
    "start_hour": 228,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2115",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_11_85,LEG_11_94,LEG_11_168,LEG_11_86"
  },
  {
    "id": 2116,
    "start_hour": 220,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2116",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_10_132,LEG_10_25"
  },
  {
    "id": 2117,
    "start_hour": 229,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D2117",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_11_8,LEG_11_105,LEG_11_159,LEG_11_75"
  },
  {
    "id": 2118,
    "start_hour": 219,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2118",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_10_30,LEG_10_29,LEG_10_26"
  },
  {
    "id": 2119,
    "start_hour": 240,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2119",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_11_167,LEG_11_20,LEG_11_15"
  },
  {
    "id": 2120,
    "start_hour": 216,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2120",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_10_165,LEG_10_64,LEG_10_73"
  },
  {
    "id": 2121,
    "start_hour": 239,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2121",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_11_71,LEG_11_33,LEG_11_149"
  },
  {
    "id": 2122,
    "start_hour": 221,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2122",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_10_6,LEG_10_2"
  },
  {
    "id": 2123,
    "start_hour": 221,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2123",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_10_145,LEG_10_154,LEG_10_28"
  },
  {
    "id": 2124,
    "start_hour": 240,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2124",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_11_82,LEG_11_189"
  },
  {
    "id": 2125,
    "start_hour": 264,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2125",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_12_178"
  },
  {
    "id": 2126,
    "start_hour": 226,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2126",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_10_98"
  },
  {
    "id": 2127,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2127",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_11_97,LEG_11_112"
  },
  {
    "id": 2128,
    "start_hour": 264,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2128",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_12_93,LEG_12_146,LEG_12_147"
  },
  {
    "id": 2129,
    "start_hour": 216,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2129",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_10_14,LEG_10_36"
  },
  {
    "id": 2130,
    "start_hour": 231,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2130",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_11_68,LEG_11_158"
  },
  {
    "id": 2131,
    "start_hour": 253,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2131",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_12_161,LEG_12_27"
  },
  {
    "id": 2132,
    "start_hour": 224,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2132",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_10_151,LEG_10_146"
  },
  {
    "id": 2133,
    "start_hour": 230,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2133",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_11_90,LEG_11_93,LEG_11_26"
  },
  {
    "id": 2134,
    "start_hour": 264,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2134",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_12_155,LEG_12_26,LEG_12_25"
  },
  {
    "id": 2135,
    "start_hour": 222,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2135",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_10_116,LEG_10_77"
  },
  {
    "id": 2136,
    "start_hour": 230,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2136",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_11_117,LEG_11_115,LEG_11_76,LEG_11_134"
  },
  {
    "id": 2137,
    "start_hour": 221,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2137",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_10_114,LEG_10_111,LEG_10_190"
  },
  {
    "id": 2138,
    "start_hour": 250,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2138",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_11_192"
  },
  {
    "id": 2139,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2139",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_19_1,LEG_19_4"
  },
  {
    "id": 2140,
    "start_hour": 432,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2140",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_19_17,LEG_19_34,LEG_19_70"
  },
  {
    "id": 2141,
    "start_hour": 455,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2141",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_20_64,LEG_20_32,LEG_20_143"
  },
  {
    "id": 2142,
    "start_hour": 439,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2142",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_19_19,LEG_19_18"
  },
  {
    "id": 2143,
    "start_hour": 446,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2143",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_20_120,LEG_20_119"
  },
  {
    "id": 2144,
    "start_hour": 432,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2144",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_19_161,LEG_19_173,LEG_19_45"
  },
  {
    "id": 2145,
    "start_hour": 456,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2145",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_20_6"
  },
  {
    "id": 2146,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2146",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_19_121,LEG_19_87"
  },
  {
    "id": 2147,
    "start_hour": 446,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2147",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_20_130,LEG_20_151"
  },
  {
    "id": 2148,
    "start_hour": 440,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2148",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_19_23,LEG_19_56"
  },
  {
    "id": 2149,
    "start_hour": 446,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2149",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_20_21"
  },
  {
    "id": 2150,
    "start_hour": 437,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2150",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_19_5,LEG_19_2"
  },
  {
    "id": 2151,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2151",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_19_84,LEG_19_127"
  },
  {
    "id": 2152,
    "start_hour": 442,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2152",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_19_93"
  },
  {
    "id": 2153,
    "start_hour": 458,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2153",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_20_92"
  },
  {
    "id": 2154,
    "start_hour": 468,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D2154",
    "gerad_crew_id": "C0034",
    "flight_ids": "LEG_21_87,LEG_21_96,LEG_21_147,LEG_21_146"
  },
  {
    "id": 2155,
    "start_hour": 437,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2155",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_19_143,LEG_19_152,LEG_19_16"
  },
  {
    "id": 2156,
    "start_hour": 455,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2156",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_20_76,LEG_20_164"
  },
  {
    "id": 2157,
    "start_hour": 488,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2157",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_21_178"
  },
  {
    "id": 2158,
    "start_hour": 494,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2158",
    "gerad_crew_id": "C0035",
    "flight_ids": "LEG_22_190,LEG_22_185,LEG_22_110,LEG_22_14"
  },
  {
    "id": 2159,
    "start_hour": 433,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2159",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_19_7,LEG_19_65,LEG_19_54"
  },
  {
    "id": 2160,
    "start_hour": 459,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2160",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_20_55,LEG_20_53"
  },
  {
    "id": 2161,
    "start_hour": 482,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2161",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_21_109,LEG_21_110,LEG_21_14"
  },
  {
    "id": 2162,
    "start_hour": 492,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2162",
    "gerad_crew_id": "C0036",
    "flight_ids": "LEG_22_87,LEG_22_96,LEG_22_171,LEG_22_88"
  },
  {
    "id": 2163,
    "start_hour": 433,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2163",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_19_72,LEG_19_104,LEG_19_51"
  },
  {
    "id": 2164,
    "start_hour": 458,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2164",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_20_39,LEG_20_175"
  },
  {
    "id": 2165,
    "start_hour": 480,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2165",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_21_179,LEG_21_144,LEG_21_57,LEG_21_48"
  },
  {
    "id": 2166,
    "start_hour": 504,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2166",
    "gerad_crew_id": "C0037",
    "flight_ids": "LEG_22_7"
  },
  {
    "id": 2167,
    "start_hour": 440,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2167",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_19_149,LEG_19_144"
  },
  {
    "id": 2168,
    "start_hour": 446,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2168",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_20_85,LEG_20_88,LEG_20_27"
  },
  {
    "id": 2169,
    "start_hour": 480,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2169",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_21_84,LEG_21_135"
  },
  {
    "id": 2170,
    "start_hour": 505,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2170",
    "gerad_crew_id": "C0038",
    "flight_ids": "LEG_22_137,LEG_22_136,LEG_22_26"
  },
  {
    "id": 2171,
    "start_hour": 49,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2171",
    "gerad_crew_id": "C0134",
    "flight_ids": "LEG_03_163,LEG_03_160"
  },
  {
    "id": 2172,
    "start_hour": 37,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2172",
    "gerad_crew_id": "C0135",
    "flight_ids": "LEG_03_137,LEG_03_40"
  },
  {
    "id": 2173,
    "start_hour": 37,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2173",
    "gerad_crew_id": "C0136",
    "flight_ids": "LEG_03_52,LEG_03_44"
  },
  {
    "id": 2174,
    "start_hour": 49,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2174",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_03_43,LEG_03_45,LEG_03_178"
  },
  {
    "id": 2175,
    "start_hour": 73,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2175",
    "gerad_crew_id": "C0137",
    "flight_ids": "LEG_04_47,LEG_04_48,LEG_04_53"
  },
  {
    "id": 2176,
    "start_hour": 49,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2176",
    "gerad_crew_id": "C0138",
    "flight_ids": "LEG_03_179,LEG_03_177,LEG_03_161,LEG_03_80"
  },
  {
    "id": 2177,
    "start_hour": 56,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2177",
    "gerad_crew_id": "C0139",
    "flight_ids": "LEG_03_109,LEG_03_110"
  },
  {
    "id": 2178,
    "start_hour": 56,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2178",
    "gerad_crew_id": "C0140",
    "flight_ids": "LEG_03_54,LEG_03_49"
  },
  {
    "id": 2179,
    "start_hour": 57,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2179",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_03_136,LEG_03_138"
  },
  {
    "id": 2180,
    "start_hour": 61,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2180",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_04_81"
  },
  {
    "id": 2181,
    "start_hour": 96,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2181",
    "gerad_crew_id": "C0141",
    "flight_ids": "LEG_05_74,LEG_05_40,LEG_05_47"
  },
  {
    "id": 2182,
    "start_hour": 51,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2182",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_03_106,LEG_03_128,LEG_03_21"
  },
  {
    "id": 2183,
    "start_hour": 73,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2183",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_04_144,LEG_04_121"
  },
  {
    "id": 2184,
    "start_hour": 96,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2184",
    "gerad_crew_id": "C0142",
    "flight_ids": "LEG_05_128,LEG_05_131,LEG_05_104"
  },
  {
    "id": 2185,
    "start_hour": 59,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2185",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_03_10"
  },
  {
    "id": 2186,
    "start_hour": 76,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2186",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_04_12,LEG_04_51"
  },
  {
    "id": 2187,
    "start_hour": 98,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2187",
    "gerad_crew_id": "C0143",
    "flight_ids": "LEG_05_39,LEG_05_174,LEG_05_9"
  },
  {
    "id": 2188,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2188",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_03_39,LEG_03_62,LEG_03_56"
  },
  {
    "id": 2189,
    "start_hour": 75,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2189",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_04_59,LEG_04_74,LEG_04_22,LEG_04_188"
  },
  {
    "id": 2190,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2190",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_05_175,LEG_05_50"
  },
  {
    "id": 2191,
    "start_hour": 122,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2191",
    "gerad_crew_id": "C0144",
    "flight_ids": "LEG_06_38,LEG_06_163,LEG_06_10"
  },
  {
    "id": 2192,
    "start_hour": 157,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2192",
    "gerad_crew_id": "C0145",
    "flight_ids": "LEG_08_52,LEG_08_44"
  },
  {
    "id": 2193,
    "start_hour": 157,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2193",
    "gerad_crew_id": "C0146",
    "flight_ids": "LEG_08_137,LEG_08_40"
  },
  {
    "id": 2194,
    "start_hour": 169,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2194",
    "gerad_crew_id": "C0147",
    "flight_ids": "LEG_08_163,LEG_08_160"
  },
  {
    "id": 2195,
    "start_hour": 174,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2195",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_08_180,LEG_08_9"
  },
  {
    "id": 2196,
    "start_hour": 182,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2196",
    "gerad_crew_id": "C0148",
    "flight_ids": "LEG_09_187,LEG_09_182,LEG_09_48,LEG_09_53"
  },
  {
    "id": 2197,
    "start_hour": 169,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2197",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_08_179,LEG_08_57,LEG_08_73"
  },
  {
    "id": 2198,
    "start_hour": 191,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2198",
    "gerad_crew_id": "C0149",
    "flight_ids": "LEG_09_71,LEG_09_70,LEG_09_184"
  },
  {
    "id": 2199,
    "start_hour": 176,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2199",
    "gerad_crew_id": "C0150",
    "flight_ids": "LEG_08_54,LEG_08_49"
  },
  {
    "id": 2200,
    "start_hour": 176,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2200",
    "gerad_crew_id": "C0151",
    "flight_ids": "LEG_08_109,LEG_08_110"
  },
  {
    "id": 2201,
    "start_hour": 171,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2201",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_08_106,LEG_08_128,LEG_08_26"
  },
  {
    "id": 2202,
    "start_hour": 192,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2202",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_09_167,LEG_09_20,LEG_09_45,LEG_09_178"
  },
  {
    "id": 2203,
    "start_hour": 217,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2203",
    "gerad_crew_id": "C0152",
    "flight_ids": "LEG_10_47,LEG_10_48,LEG_10_53"
  },
  {
    "id": 2204,
    "start_hour": 179,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2204",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_08_10"
  },
  {
    "id": 2205,
    "start_hour": 196,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2205",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_09_12,LEG_09_188"
  },
  {
    "id": 2206,
    "start_hour": 218,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2206",
    "gerad_crew_id": "C0153",
    "flight_ids": "LEG_10_181,LEG_10_42,LEG_10_50"
  },
  {
    "id": 2207,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2207",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_08_43,LEG_08_15,LEG_08_170"
  },
  {
    "id": 2208,
    "start_hour": 193,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2208",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_09_113,LEG_09_87"
  },
  {
    "id": 2209,
    "start_hour": 218,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2209",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_10_89,LEG_10_170"
  },
  {
    "id": 2210,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2210",
    "gerad_crew_id": "C0154",
    "flight_ids": "LEG_11_181,LEG_11_180,LEG_11_9"
  },
  {
    "id": 2211,
    "start_hour": 170,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2211",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_08_172"
  },
  {
    "id": 2212,
    "start_hour": 200,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2212",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_09_174"
  },
  {
    "id": 2213,
    "start_hour": 206,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2213",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_10_187,LEG_10_182,LEG_10_79,LEG_10_108,LEG_10_185"
  },
  {
    "id": 2214,
    "start_hour": 240,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2214",
    "gerad_crew_id": "C0155",
    "flight_ids": "LEG_11_60,LEG_11_70,LEG_11_184"
  },
  {
    "id": 2215,
    "start_hour": 177,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2215",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_08_136,LEG_08_138"
  },
  {
    "id": 2216,
    "start_hour": 181,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2216",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_09_81"
  },
  {
    "id": 2217,
    "start_hour": 216,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2217",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_10_78,LEG_10_140,LEG_10_55,LEG_10_178"
  },
  {
    "id": 2218,
    "start_hour": 241,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2218",
    "gerad_crew_id": "C0156",
    "flight_ids": "LEG_11_47,LEG_11_183,LEG_11_186"
  },
  {
    "id": 2219,
    "start_hour": 388,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2219",
    "gerad_crew_id": "C0039",
    "flight_ids": "LEG_17_136,LEG_17_26"
  },
  {
    "id": 2220,
    "start_hour": 389,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2220",
    "gerad_crew_id": "C0040",
    "flight_ids": "LEG_17_149,LEG_17_158"
  },
  {
    "id": 2221,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2221",
    "gerad_crew_id": "C0041",
    "flight_ids": "LEG_17_1,LEG_17_5"
  },
  {
    "id": 2222,
    "start_hour": 387,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2222",
    "gerad_crew_id": "C0042",
    "flight_ids": "LEG_17_147,LEG_17_146"
  },
  {
    "id": 2223,
    "start_hour": 375,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2223",
    "gerad_crew_id": "C0043",
    "flight_ids": "LEG_17_3,LEG_17_24"
  },
  {
    "id": 2224,
    "start_hour": 386,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2224",
    "gerad_crew_id": "C0044",
    "flight_ids": "LEG_17_97,LEG_17_101"
  },
  {
    "id": 2225,
    "start_hour": 393,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2225",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_17_94,LEG_17_71"
  },
  {
    "id": 2226,
    "start_hour": 398,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2226",
    "gerad_crew_id": "C0045",
    "flight_ids": "LEG_18_25"
  },
  {
    "id": 2227,
    "start_hour": 384,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2227",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_17_169,LEG_17_181,LEG_17_48"
  },
  {
    "id": 2228,
    "start_hour": 408,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2228",
    "gerad_crew_id": "C0046",
    "flight_ids": "LEG_18_7"
  },
  {
    "id": 2229,
    "start_hour": 389,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2229",
    "gerad_crew_id": "C0047",
    "flight_ids": "LEG_17_6,LEG_17_2"
  },
  {
    "id": 2230,
    "start_hour": 391,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2230",
    "gerad_crew_id": "C0048",
    "flight_ids": "LEG_17_78,LEG_17_138"
  },
  {
    "id": 2231,
    "start_hour": 390,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2231",
    "gerad_crew_id": "C0049",
    "flight_ids": "LEG_17_120,LEG_17_79"
  },
  {
    "id": 2232,
    "start_hour": 390,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2232",
    "gerad_crew_id": "C0050",
    "flight_ids": "LEG_17_124,LEG_17_93"
  },
  {
    "id": 2233,
    "start_hour": 391,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2233",
    "gerad_crew_id": "C0051",
    "flight_ids": "LEG_17_19,LEG_17_18"
  },
  {
    "id": 2234,
    "start_hour": 396,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2234",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_17_29"
  },
  {
    "id": 2235,
    "start_hour": 408,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2235",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_18_84,LEG_18_135"
  },
  {
    "id": 2236,
    "start_hour": 433,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2236",
    "gerad_crew_id": "C0052",
    "flight_ids": "LEG_19_134,LEG_19_79,LEG_19_80"
  },
  {
    "id": 2237,
    "start_hour": 384,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2237",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_17_15,LEG_17_38"
  },
  {
    "id": 2238,
    "start_hour": 399,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2238",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_18_70,LEG_18_162"
  },
  {
    "id": 2239,
    "start_hour": 421,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2239",
    "gerad_crew_id": "C0053",
    "flight_ids": "LEG_19_169,LEG_19_27"
  },
  {
    "id": 2240,
    "start_hour": 396,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2240",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_17_4"
  },
  {
    "id": 2241,
    "start_hour": 410,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2241",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_18_143,LEG_18_110,LEG_18_14,LEG_18_27"
  },
  {
    "id": 2242,
    "start_hour": 432,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2242",
    "gerad_crew_id": "C0054",
    "flight_ids": "LEG_19_163,LEG_19_26,LEG_19_25"
  },
  {
    "id": 2243,
    "start_hour": 396,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2243",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_17_22"
  },
  {
    "id": 2244,
    "start_hour": 409,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2244",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_18_148,LEG_18_118,LEG_18_115,LEG_18_116"
  },
  {
    "id": 2245,
    "start_hour": 432,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2245",
    "gerad_crew_id": "C0055",
    "flight_ids": "LEG_19_96,LEG_19_154,LEG_19_155"
  },
  {
    "id": 2246,
    "start_hour": 396,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2246",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_17_194"
  },
  {
    "id": 2247,
    "start_hour": 418,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2247",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_18_196"
  },
  {
    "id": 2248,
    "start_hour": 422,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D2248",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_19_145,LEG_19_150,LEG_19_24"
  },
  {
    "id": 2249,
    "start_hour": 456,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2249",
    "gerad_crew_id": "C0056",
    "flight_ids": "LEG_20_78,LEG_20_109,LEG_20_106"
  },
  {
    "id": 2250,
    "start_hour": 394,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2250",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_17_131"
  },
  {
    "id": 2251,
    "start_hour": 410,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2251",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_18_123,LEG_18_175"
  },
  {
    "id": 2252,
    "start_hour": 432,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2252",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_19_171,LEG_19_13"
  },
  {
    "id": 2253,
    "start_hour": 458,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2253",
    "gerad_crew_id": "C0057",
    "flight_ids": "LEG_20_7"
  },
  {
    "id": 2254,
    "start_hour": 394,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2254",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_17_100"
  },
  {
    "id": 2255,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2255",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_18_99,LEG_18_131"
  },
  {
    "id": 2256,
    "start_hour": 434,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2256",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_19_120,LEG_19_184"
  },
  {
    "id": 2257,
    "start_hour": 456,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2257",
    "gerad_crew_id": "C0058",
    "flight_ids": "LEG_20_180"
  },
  {
    "id": 2258,
    "start_hour": 396,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2258",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_17_157"
  },
  {
    "id": 2259,
    "start_hour": 409,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2259",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_18_34,LEG_18_125"
  },
  {
    "id": 2260,
    "start_hour": 432,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2260",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_19_130,LEG_19_132"
  },
  {
    "id": 2261,
    "start_hour": 457,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2261",
    "gerad_crew_id": "C0059",
    "flight_ids": "LEG_20_128,LEG_20_79,LEG_20_80"
  },
  {
    "id": 2262,
    "start_hour": 392,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2262",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_17_155,LEG_17_150"
  },
  {
    "id": 2263,
    "start_hour": 398,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2263",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_18_139,LEG_18_161,LEG_18_22"
  },
  {
    "id": 2264,
    "start_hour": 433,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2264",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_19_142,LEG_19_115,LEG_19_112,LEG_19_113"
  },
  {
    "id": 2265,
    "start_hour": 456,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2265",
    "gerad_crew_id": "C0060",
    "flight_ids": "LEG_20_96,LEG_20_149,LEG_20_150"
  },
  {
    "id": 2266,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2266",
    "gerad_crew_id": "C0061",
    "flight_ids": "LEG_04_95,LEG_04_99"
  },
  {
    "id": 2267,
    "start_hour": 63,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2267",
    "gerad_crew_id": "C0062",
    "flight_ids": "LEG_04_3,LEG_04_23"
  },
  {
    "id": 2268,
    "start_hour": 72,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2268",
    "gerad_crew_id": "C0063",
    "flight_ids": "LEG_04_165,LEG_04_166"
  },
  {
    "id": 2269,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2269",
    "gerad_crew_id": "C0064",
    "flight_ids": "LEG_04_1,LEG_04_5"
  },
  {
    "id": 2270,
    "start_hour": 80,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2270",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_04_151,LEG_04_146"
  },
  {
    "id": 2271,
    "start_hour": 86,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2271",
    "gerad_crew_id": "C0065",
    "flight_ids": "LEG_05_124,LEG_05_123"
  },
  {
    "id": 2272,
    "start_hour": 81,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2272",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_04_92,LEG_04_69"
  },
  {
    "id": 2273,
    "start_hour": 86,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2273",
    "gerad_crew_id": "C0066",
    "flight_ids": "LEG_05_22"
  },
  {
    "id": 2274,
    "start_hour": 78,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2274",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_04_116,LEG_04_77"
  },
  {
    "id": 2275,
    "start_hour": 85,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2275",
    "gerad_crew_id": "C0067",
    "flight_ids": "LEG_05_8,LEG_05_101,LEG_05_102,LEG_05_127"
  },
  {
    "id": 2276,
    "start_hour": 72,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2276",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_04_14,LEG_04_36,LEG_04_73"
  },
  {
    "id": 2277,
    "start_hour": 95,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2277",
    "gerad_crew_id": "C0068",
    "flight_ids": "LEG_05_67,LEG_05_30,LEG_05_145"
  },
  {
    "id": 2278,
    "start_hour": 76,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2278",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_04_132,LEG_04_25,LEG_04_26"
  },
  {
    "id": 2279,
    "start_hour": 96,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2279",
    "gerad_crew_id": "C0069",
    "flight_ids": "LEG_05_161,LEG_05_26,LEG_05_25"
  },
  {
    "id": 2280,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2280",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_04_88,LEG_04_126"
  },
  {
    "id": 2281,
    "start_hour": 86,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D2281",
    "gerad_crew_id": "C0070",
    "flight_ids": "LEG_05_134,LEG_05_154"
  },
  {
    "id": 2282,
    "start_hour": 75,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2282",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_04_143,LEG_04_142"
  },
  {
    "id": 2283,
    "start_hour": 84,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D2283",
    "gerad_crew_id": "C0071",
    "flight_ids": "LEG_05_80,LEG_05_89,LEG_05_139,LEG_05_138"
  },
  {
    "id": 2284,
    "start_hour": 77,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2284",
    "gerad_crew_id": "C0072",
    "flight_ids": "LEG_04_6,LEG_04_2"
  },
  {
    "id": 2285,
    "start_hour": 78,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2285",
    "gerad_crew_id": "C0073",
    "flight_ids": "LEG_04_120,LEG_04_91"
  },
  {
    "id": 2286,
    "start_hour": 77,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2286",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_04_145,LEG_04_154,LEG_04_28"
  },
  {
    "id": 2287,
    "start_hour": 96,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2287",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_05_77,LEG_05_130"
  },
  {
    "id": 2288,
    "start_hour": 121,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2288",
    "gerad_crew_id": "C0074",
    "flight_ids": "LEG_06_120,LEG_06_131,LEG_06_140"
  },
  {
    "id": 2289,
    "start_hour": 77,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2289",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_04_114,LEG_04_111,LEG_04_190"
  },
  {
    "id": 2290,
    "start_hour": 106,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2290",
    "gerad_crew_id": "C0075",
    "flight_ids": "LEG_05_185"
  },
  {
    "id": 2291,
    "start_hour": 79,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2291",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_04_18,LEG_04_17"
  },
  {
    "id": 2292,
    "start_hour": 86,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2292",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_05_85,LEG_05_88,LEG_05_137"
  },
  {
    "id": 2293,
    "start_hour": 120,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2293",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_06_109,LEG_06_108"
  },
  {
    "id": 2294,
    "start_hour": 144,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2294",
    "gerad_crew_id": "C0076",
    "flight_ids": "LEG_07_128"
  },
  {
    "id": 2295,
    "start_hour": 80,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2295",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_04_27,LEG_04_58"
  },
  {
    "id": 2296,
    "start_hour": 87,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2296",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_05_65"
  },
  {
    "id": 2297,
    "start_hour": 121,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2297",
    "gerad_crew_id": "C0077",
    "flight_ids": "LEG_06_89,LEG_06_107,LEG_06_80"
  },
  {
    "id": 2298,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2298",
    "gerad_crew_id": "C0078",
    "flight_ids": "LEG_03_95,LEG_03_99"
  },
  {
    "id": 2299,
    "start_hour": 39,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2299",
    "gerad_crew_id": "C0079",
    "flight_ids": "LEG_03_3,LEG_03_23"
  },
  {
    "id": 2300,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2300",
    "gerad_crew_id": "C0080",
    "flight_ids": "LEG_03_1,LEG_03_5"
  },
  {
    "id": 2301,
    "start_hour": 57,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2301",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_03_92,LEG_03_69"
  },
  {
    "id": 2302,
    "start_hour": 62,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2302",
    "gerad_crew_id": "C0081",
    "flight_ids": "LEG_04_24"
  },
  {
    "id": 2303,
    "start_hour": 51,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D2303",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_03_143,LEG_03_142"
  },
  {
    "id": 2304,
    "start_hour": 60,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2304",
    "gerad_crew_id": "C0082",
    "flight_ids": "LEG_04_85,LEG_04_94,LEG_04_168,LEG_04_86"
  },
  {
    "id": 2305,
    "start_hour": 51,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2305",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_03_30,LEG_03_29,LEG_03_26"
  },
  {
    "id": 2306,
    "start_hour": 72,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2306",
    "gerad_crew_id": "C0083",
    "flight_ids": "LEG_04_167,LEG_04_30,LEG_04_29"
  },
  {
    "id": 2307,
    "start_hour": 54,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2307",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_03_116,LEG_03_77"
  },
  {
    "id": 2308,
    "start_hour": 61,
    "duration_hours": 20,
    "required_skill": "A320",
    "gerad_duty_id": "D2308",
    "gerad_crew_id": "C0084",
    "flight_ids": "LEG_04_8,LEG_04_105,LEG_04_159,LEG_04_75"
  },
  {
    "id": 2309,
    "start_hour": 53,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2309",
    "gerad_crew_id": "C0085",
    "flight_ids": "LEG_03_6,LEG_03_2"
  },
  {
    "id": 2310,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2310",
    "gerad_crew_id": "C0086",
    "flight_ids": "LEG_03_120,LEG_03_91"
  },
  {
    "id": 2311,
    "start_hour": 38,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2311",
    "gerad_crew_id": "C0087",
    "flight_ids": "LEG_03_117,LEG_03_115,LEG_03_76,LEG_03_134"
  },
  {
    "id": 2312,
    "start_hour": 36,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2312",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_03_123,LEG_03_166,LEG_03_27,LEG_03_58"
  },
  {
    "id": 2313,
    "start_hour": 97,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2313",
    "gerad_crew_id": "C0088",
    "flight_ids": "LEG_05_93,LEG_05_113,LEG_05_110"
  },
  {
    "id": 2314,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2314",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_03_147,LEG_03_152,LEG_03_153"
  },
  {
    "id": 2315,
    "start_hour": 73,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2315",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_04_32,LEG_04_96,LEG_04_169,LEG_04_112"
  },
  {
    "id": 2316,
    "start_hour": 96,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2316",
    "gerad_crew_id": "C0089",
    "flight_ids": "LEG_05_95,LEG_05_152,LEG_05_153"
  },
  {
    "id": 2317,
    "start_hour": 48,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2317",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_03_14,LEG_03_36"
  },
  {
    "id": 2318,
    "start_hour": 63,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2318",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_04_68,LEG_04_158"
  },
  {
    "id": 2319,
    "start_hour": 85,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2319",
    "gerad_crew_id": "C0090",
    "flight_ids": "LEG_05_167,LEG_05_27"
  },
  {
    "id": 2320,
    "start_hour": 55,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2320",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_03_18,LEG_03_17"
  },
  {
    "id": 2321,
    "start_hour": 62,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2321",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_04_125,LEG_04_124,LEG_04_4"
  },
  {
    "id": 2322,
    "start_hour": 96,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2322",
    "gerad_crew_id": "C0091",
    "flight_ids": "LEG_05_6"
  },
  {
    "id": 2323,
    "start_hour": 56,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2323",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_03_151,LEG_03_146"
  },
  {
    "id": 2324,
    "start_hour": 62,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D2324",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_04_90,LEG_04_93,LEG_04_141"
  },
  {
    "id": 2325,
    "start_hour": 96,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2325",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_05_121,LEG_05_120"
  },
  {
    "id": 2326,
    "start_hour": 120,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2326",
    "gerad_crew_id": "C0092",
    "flight_ids": "LEG_06_116,LEG_06_101,LEG_06_99"
  },
  {
    "id": 2327,
    "start_hour": 53,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2327",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_03_145,LEG_03_154,LEG_03_28"
  },
  {
    "id": 2328,
    "start_hour": 72,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2328",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_04_82,LEG_04_171"
  },
  {
    "id": 2329,
    "start_hour": 96,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2329",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_05_169,LEG_05_13"
  },
  {
    "id": 2330,
    "start_hour": 122,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D2330",
    "gerad_crew_id": "C0093",
    "flight_ids": "LEG_06_9"
  },
  {
    "id": 2331,
    "start_hour": 54,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2331",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_03_88,LEG_03_126"
  },
  {
    "id": 2332,
    "start_hour": 62,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D2332",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_04_135,LEG_04_157"
  },
  {
    "id": 2333,
    "start_hour": 84,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2333",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_05_122,LEG_05_158,LEG_05_111"
  },
  {
    "id": 2334,
    "start_hour": 120,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2334",
    "gerad_crew_id": "C0094",
    "flight_ids": "LEG_06_90,LEG_06_141,LEG_06_142"
  },
  {
    "id": 2335,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2335",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_03_125,LEG_03_124,LEG_03_190"
  },
  {
    "id": 2336,
    "start_hour": 82,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2336",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_04_192"
  },
  {
    "id": 2337,
    "start_hour": 86,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2337",
    "gerad_crew_id": "C0095",
    "flight_ids": "LEG_05_116,LEG_05_114,LEG_05_72,LEG_05_133"
  },
  {
    "id": 2338,
    "start_hour": 58,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2338",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_03_98"
  },
  {
    "id": 2339,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2339",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_04_97,LEG_04_87"
  },
  {
    "id": 2340,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2340",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_05_84,LEG_05_146"
  },
  {
    "id": 2341,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2341",
    "gerad_crew_id": "C0096",
    "flight_ids": "LEG_06_134,LEG_06_154,LEG_06_105"
  },
  {
    "id": 2342,
    "start_hour": 13,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2342",
    "gerad_crew_id": "C0157",
    "flight_ids": "LEG_02_136,LEG_02_40"
  },
  {
    "id": 2343,
    "start_hour": 25,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2343",
    "gerad_crew_id": "C0158",
    "flight_ids": "LEG_02_43,LEG_02_45"
  },
  {
    "id": 2344,
    "start_hour": 13,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2344",
    "gerad_crew_id": "C0159",
    "flight_ids": "LEG_02_52,LEG_02_44"
  },
  {
    "id": 2345,
    "start_hour": 25,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2345",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_02_162,LEG_02_159,LEG_02_177"
  },
  {
    "id": 2346,
    "start_hour": 49,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2346",
    "gerad_crew_id": "C0160",
    "flight_ids": "LEG_03_47,LEG_03_183,LEG_03_186"
  },
  {
    "id": 2347,
    "start_hour": 35,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D2347",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_02_187"
  },
  {
    "id": 2348,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2348",
    "gerad_crew_id": "C0161",
    "flight_ids": "LEG_03_181,LEG_03_42,LEG_03_50"
  },
  {
    "id": 2349,
    "start_hour": 25,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2349",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_02_178,LEG_02_57,LEG_02_73"
  },
  {
    "id": 2350,
    "start_hour": 47,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2350",
    "gerad_crew_id": "C0162",
    "flight_ids": "LEG_03_71,LEG_03_70,LEG_03_184"
  },
  {
    "id": 2351,
    "start_hour": 32,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2351",
    "gerad_crew_id": "C0163",
    "flight_ids": "LEG_02_54,LEG_02_49"
  },
  {
    "id": 2352,
    "start_hour": 32,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2352",
    "gerad_crew_id": "C0164",
    "flight_ids": "LEG_02_108,LEG_02_109"
  },
  {
    "id": 2353,
    "start_hour": 28,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2353",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_02_175,LEG_02_38,LEG_02_56"
  },
  {
    "id": 2354,
    "start_hour": 51,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2354",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_03_59,LEG_03_74,LEG_03_22,LEG_03_188"
  },
  {
    "id": 2355,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2355",
    "gerad_crew_id": "C0165",
    "flight_ids": "LEG_04_181,LEG_04_42,LEG_04_50"
  },
  {
    "id": 2356,
    "start_hour": 35,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2356",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_02_10"
  },
  {
    "id": 2357,
    "start_hour": 52,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2357",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_03_12,LEG_03_51"
  },
  {
    "id": 2358,
    "start_hour": 74,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2358",
    "gerad_crew_id": "C0166",
    "flight_ids": "LEG_04_41,LEG_04_180,LEG_04_9"
  },
  {
    "id": 2359,
    "start_hour": 13,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2359",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_02_172,LEG_02_31"
  },
  {
    "id": 2360,
    "start_hour": 38,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2360",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_03_90,LEG_03_93,LEG_03_4"
  },
  {
    "id": 2361,
    "start_hour": 74,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2361",
    "gerad_crew_id": "C0167",
    "flight_ids": "LEG_04_139,LEG_04_183,LEG_04_186"
  },
  {
    "id": 2362,
    "start_hour": 27,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2362",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_02_105,LEG_02_127,LEG_02_86"
  },
  {
    "id": 2363,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2363",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_03_89,LEG_03_171"
  },
  {
    "id": 2364,
    "start_hour": 72,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2364",
    "gerad_crew_id": "C0168",
    "flight_ids": "LEG_04_60,LEG_04_70,LEG_04_184"
  },
  {
    "id": 2365,
    "start_hour": 33,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2365",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_02_135,LEG_02_137"
  },
  {
    "id": 2366,
    "start_hour": 37,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2366",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_03_81"
  },
  {
    "id": 2367,
    "start_hour": 72,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2367",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_04_78,LEG_04_140,LEG_04_55,LEG_04_46"
  },
  {
    "id": 2368,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2368",
    "gerad_crew_id": "C0169",
    "flight_ids": "LEG_05_136,LEG_05_177,LEG_05_179"
  },
  {
    "id": 2369,
    "start_hour": 26,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2369",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_02_171"
  },
  {
    "id": 2370,
    "start_hour": 56,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2370",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_03_174"
  },
  {
    "id": 2371,
    "start_hour": 62,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2371",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_04_187,LEG_04_182,LEG_04_79,LEG_04_108,LEG_04_185"
  },
  {
    "id": 2372,
    "start_hour": 96,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2372",
    "gerad_crew_id": "C0170",
    "flight_ids": "LEG_05_57,LEG_05_58,LEG_05_106"
  },
  {
    "id": 2373,
    "start_hour": 656,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2373",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_28_21,LEG_28_19"
  },
  {
    "id": 2374,
    "start_hour": 663,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2374",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_29_86,LEG_29_89,LEG_29_174"
  },
  {
    "id": 2375,
    "start_hour": 698,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2375",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_30_140,LEG_30_30,LEG_30_108"
  },
  {
    "id": 2376,
    "start_hour": 724,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2376",
    "gerad_crew_id": "C0097",
    "flight_ids": "LEG_31_159,LEG_31_126"
  },
  {
    "id": 2377,
    "start_hour": 653,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2377",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_28_24,LEG_28_65"
  },
  {
    "id": 2378,
    "start_hour": 661,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2378",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_29_111,LEG_29_17,LEG_29_158"
  },
  {
    "id": 2379,
    "start_hour": 685,
    "duration_hours": 19,
    "required_skill": "A320",
    "gerad_duty_id": "D2379",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_30_165,LEG_30_199,LEG_30_169"
  },
  {
    "id": 2380,
    "start_hour": 709,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2380",
    "gerad_crew_id": "C0098",
    "flight_ids": "LEG_31_136,LEG_31_114,LEG_31_116,LEG_31_118"
  },
  {
    "id": 2381,
    "start_hour": 650,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2381",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_28_103,LEG_28_176,LEG_28_7"
  },
  {
    "id": 2382,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2382",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_29_60,LEG_29_10,LEG_29_54,LEG_29_57"
  },
  {
    "id": 2383,
    "start_hour": 697,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2383",
    "gerad_crew_id": "C0099",
    "flight_ids": "LEG_30_148,LEG_30_69,LEG_30_27"
  },
  {
    "id": 2384,
    "start_hour": 660,
    "duration_hours": 5,
    "required_skill": "A319",
    "gerad_duty_id": "D2384",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_28_89"
  },
  {
    "id": 2385,
    "start_hour": 675,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2385",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_29_96,LEG_29_125"
  },
  {
    "id": 2386,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2386",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_30_122,LEG_30_81"
  },
  {
    "id": 2387,
    "start_hour": 723,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2387",
    "gerad_crew_id": "C0100",
    "flight_ids": "LEG_31_85,LEG_31_22,LEG_31_20"
  },
  {
    "id": 2388,
    "start_hour": 654,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2388",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_28_110,LEG_28_109"
  },
  {
    "id": 2389,
    "start_hour": 661,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2389",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_29_117,LEG_29_196,LEG_29_142"
  },
  {
    "id": 2390,
    "start_hour": 699,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2390",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_30_144,LEG_30_173"
  },
  {
    "id": 2391,
    "start_hour": 722,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D2391",
    "gerad_crew_id": "C0101",
    "flight_ids": "LEG_31_141,LEG_31_36,LEG_31_41"
  },
  {
    "id": 2392,
    "start_hour": 649,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D2392",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_28_168,LEG_28_194,LEG_28_146"
  },
  {
    "id": 2393,
    "start_hour": 673,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D2393",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_29_182,LEG_29_183"
  },
  {
    "id": 2394,
    "start_hour": 696,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2394",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_30_92,LEG_30_134"
  },
  {
    "id": 2395,
    "start_hour": 723,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D2395",
    "gerad_crew_id": "C0102",
    "flight_ids": "LEG_31_138"
  },
  {
    "id": 2396,
    "start_hour": 654,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2396",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_28_35,LEG_28_40"
  },
  {
    "id": 2397,
    "start_hour": 661,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2397",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_29_163"
  },
  {
    "id": 2398,
    "start_hour": 685,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2398",
    "gerad_crew_id": "C0103",
    "flight_ids": "LEG_30_47"
  },
  {
    "id": 2399,
    "start_hour": 651,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D2399",
    "gerad_crew_id": "C0104",
    "flight_ids": "LEG_28_92,LEG_28_91"
  },
  {
    "id": 2400,
    "start_hour": 657,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2400",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_28_26"
  },
  {
    "id": 2401,
    "start_hour": 660,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2401",
    "gerad_crew_id": "C0105",
    "flight_ids": "LEG_29_72,LEG_29_24"
  },
  {
    "id": 2402,
    "start_hour": 654,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2402",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_28_77,LEG_28_80"
  },
  {
    "id": 2403,
    "start_hour": 662,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2403",
    "gerad_crew_id": "C0106",
    "flight_ids": "LEG_29_4,LEG_29_13,LEG_29_104,LEG_29_175"
  },
  {
    "id": 2404,
    "start_hour": 656,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D2404",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_28_37,LEG_28_152"
  },
  {
    "id": 2405,
    "start_hour": 663,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2405",
    "gerad_crew_id": "C0107",
    "flight_ids": "LEG_29_143,LEG_29_140"
  },
  {
    "id": 2406,
    "start_hour": 654,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2406",
    "gerad_crew_id": "C0108",
    "flight_ids": "LEG_28_1,LEG_28_3"
  },
  {
    "id": 2407,
    "start_hour": 652,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2407",
    "gerad_crew_id": "C0109",
    "flight_ids": "LEG_28_151,LEG_28_149"
  },
  {
    "id": 2408,
    "start_hour": 652,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D2408",
    "gerad_crew_id": "C0110",
    "flight_ids": "LEG_28_157,LEG_28_160"
  },
  {
    "id": 2409,
    "start_hour": 656,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2409",
    "gerad_crew_id": "C0111",
    "flight_ids": "LEG_28_167,LEG_28_118"
  },
  {
    "id": 2410,
    "start_hour": 655,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2410",
    "gerad_crew_id": "C0112",
    "flight_ids": "LEG_28_120,LEG_28_193"
  },
  {
    "id": 2411,
    "start_hour": 649,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2411",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_28_18,LEG_28_45,LEG_28_203"
  },
  {
    "id": 2412,
    "start_hour": 676,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2412",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_29_202,LEG_29_73"
  },
  {
    "id": 2413,
    "start_hour": 700,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2413",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_30_66,LEG_30_46,LEG_30_44"
  },
  {
    "id": 2414,
    "start_hour": 723,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2414",
    "gerad_crew_id": "C0113",
    "flight_ids": "LEG_31_43,LEG_31_26"
  },
  {
    "id": 2415,
    "start_hour": 639,
    "duration_hours": 21,
    "required_skill": "A321",
    "gerad_duty_id": "D2415",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_28_129,LEG_28_132"
  },
  {
    "id": 2416,
    "start_hour": 661,
    "duration_hours": 22,
    "required_skill": "A321",
    "gerad_duty_id": "D2416",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_29_139,LEG_29_83"
  },
  {
    "id": 2417,
    "start_hour": 685,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2417",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_30_162"
  },
  {
    "id": 2418,
    "start_hour": 709,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2418",
    "gerad_crew_id": "C0114",
    "flight_ids": "LEG_31_48"
  },
  {
    "id": 2419,
    "start_hour": 615,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2419",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_27_60,LEG_27_127"
  },
  {
    "id": 2420,
    "start_hour": 636,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D2420",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_28_5,LEG_28_8"
  },
  {
    "id": 2421,
    "start_hour": 660,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2421",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_29_184,LEG_29_193,LEG_29_181"
  },
  {
    "id": 2422,
    "start_hour": 685,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2422",
    "gerad_crew_id": "C0219",
    "flight_ids": "LEG_30_147"
  },
  {
    "id": 2423,
    "start_hour": 636,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D2423",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_27_117"
  },
  {
    "id": 2424,
    "start_hour": 639,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2424",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_28_117,LEG_28_116"
  },
  {
    "id": 2425,
    "start_hour": 661,
    "duration_hours": 26,
    "required_skill": "A320",
    "gerad_duty_id": "D2425",
    "gerad_crew_id": "C0220",
    "flight_ids": "LEG_29_80,LEG_29_46,LEG_29_47,LEG_29_45"
  },
  {
    "id": 2426,
    "start_hour": 614,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2426",
    "gerad_crew_id": "C0221",
    "flight_ids": "LEG_27_39,LEG_27_162"
  },
  {
    "id": 2427,
    "start_hour": 627,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2427",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_27_161,LEG_27_83,LEG_27_48"
  },
  {
    "id": 2428,
    "start_hour": 648,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D2428",
    "gerad_crew_id": "C0222",
    "flight_ids": "LEG_28_101,LEG_28_14,LEG_28_147"
  },
  {
    "id": 2429,
    "start_hour": 614,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D2429",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_27_22,LEG_27_118,LEG_27_165"
  },
  {
    "id": 2430,
    "start_hour": 640,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D2430",
    "gerad_crew_id": "C0223",
    "flight_ids": "LEG_28_2"
  },
  {
    "id": 2431,
    "start_hour": 631,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2431",
    "gerad_crew_id": "C0184",
    "flight_ids": "LEG_27_157,LEG_27_158"
  },
  {
    "id": 2432,
    "start_hour": 627,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2432",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_27_57,LEG_27_25,LEG_27_30"
  },
  {
    "id": 2433,
    "start_hour": 654,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D2433",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_28_33,LEG_28_28"
  },
  {
    "id": 2434,
    "start_hour": 661,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2434",
    "gerad_crew_id": "C0185",
    "flight_ids": "LEG_29_75"
  },
  {
    "id": 2435,
    "start_hour": 631,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D2435",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_27_160,LEG_27_146"
  },
  {
    "id": 2436,
    "start_hour": 638,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D2436",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_28_50"
  },
  {
    "id": 2437,
    "start_hour": 672,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2437",
    "gerad_crew_id": "C0186",
    "flight_ids": "LEG_29_134,LEG_29_128,LEG_29_74"
  },
  {
    "id": 2438,
    "start_hour": 112,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2438",
    "gerad_crew_id": "C0115",
    "flight_ids": "LEG_06_1,LEG_06_83"
  },
  {
    "id": 2439,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2439",
    "gerad_crew_id": "C0116",
    "flight_ids": "LEG_06_3,LEG_06_6"
  },
  {
    "id": 2440,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D2440",
    "gerad_crew_id": "C0117",
    "flight_ids": "LEG_06_84,LEG_06_88"
  },
  {
    "id": 2441,
    "start_hour": 123,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D2441",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_06_130,LEG_06_129"
  },
  {
    "id": 2442,
    "start_hour": 132,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D2442",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_07_85,LEG_07_94,LEG_07_167,LEG_07_86"
  },
  {
    "id": 2443,
    "start_hour": 129,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2443",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_06_81,LEG_06_60"
  },
  {
    "id": 2444,
    "start_hour": 134,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D2444",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_07_24"
  },
  {
    "id": 2445,
    "start_hour": 123,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2445",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_06_28,LEG_06_27,LEG_06_24"
  },
  {
    "id": 2446,
    "start_hour": 144,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2446",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_07_166,LEG_07_30,LEG_07_29"
  },
  {
    "id": 2447,
    "start_hour": 125,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D2447",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_06_0,LEG_06_2"
  },
  {
    "id": 2448,
    "start_hour": 133,
    "duration_hours": 20,
    "required_skill": "A321",
    "gerad_duty_id": "D2448",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_07_8,LEG_07_104,LEG_07_158,LEG_07_75"
  },
  {
    "id": 2449,
    "start_hour": 124,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2449",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_06_119,LEG_06_23"
  },
  {
    "id": 2450,
    "start_hour": 132,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D2450",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_07_13,LEG_07_0,LEG_07_131,LEG_07_25"
  },
  {
    "id": 2451,
    "start_hour": 110,
    "duration_hours": 24,
    "required_skill": "A320",
    "gerad_duty_id": "D2451",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_06_104,LEG_06_102,LEG_06_67,LEG_06_121"
  },
  {
    "id": 2452,
    "start_hour": 125,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2452",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_06_7,LEG_06_4"
  },
  {
    "id": 2453,
    "start_hour": 110,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2453",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_06_112,LEG_06_111,LEG_06_139"
  },
  {
    "id": 2454,
    "start_hour": 145,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2454",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_07_32,LEG_07_96,LEG_07_168,LEG_07_111"
  },
  {
    "id": 2455,
    "start_hour": 168,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2455",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_08_101,LEG_08_155,LEG_08_156"
  },
  {
    "id": 2456,
    "start_hour": 120,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D2456",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_06_149,LEG_06_56"
  },
  {
    "id": 2457,
    "start_hour": 135,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D2457",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_07_68,LEG_07_157"
  },
  {
    "id": 2458,
    "start_hour": 157,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D2458",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_08_173,LEG_08_31"
  },
  {
    "id": 2459,
    "start_hour": 127,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D2459",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_06_17,LEG_06_16"
  },
  {
    "id": 2460,
    "start_hour": 134,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D2460",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_07_124,LEG_07_123,LEG_07_152"
  },
  {
    "id": 2461,
    "start_hour": 169,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2461",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_08_32,LEG_08_96,LEG_08_169,LEG_08_112"
  },
  {
    "id": 2462,
    "start_hour": 192,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2462",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_09_101,LEG_09_155,LEG_09_156"
  },
  {
    "id": 2463,
    "start_hour": 123,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D2463",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_06_151,LEG_06_75,LEG_06_128"
  },
  {
    "id": 2464,
    "start_hour": 144,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2464",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_07_121,LEG_07_120"
  },
  {
    "id": 2465,
    "start_hour": 168,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D2465",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_08_129,LEG_08_131"
  },
  {
    "id": 2466,
    "start_hour": 193,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2466",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_09_133,LEG_09_83,LEG_09_84"
  },
  {
    "id": 2467,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D2467",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_06_77,LEG_06_113"
  },
  {
    "id": 2468,
    "start_hour": 134,
    "duration_hours": 26,
    "required_skill": "A321",
    "gerad_duty_id": "D2468",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_07_134,LEG_07_156,LEG_07_189"
  },
  {
    "id": 2469,
    "start_hour": 178,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D2469",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_08_192"
  },
  {
    "id": 2470,
    "start_hour": 128,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D2470",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_06_137,LEG_06_132"
  },
  {
    "id": 2471,
    "start_hour": 134,
    "duration_hours": 24,
    "required_skill": "A319",
    "gerad_duty_id": "D2471",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_07_90,LEG_07_93,LEG_07_126"
  },
  {
    "id": 2472,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2472",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_08_119,LEG_08_150"
  },
  {
    "id": 2473,
    "start_hour": 194,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D2473",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_09_148,LEG_09_116,LEG_09_77"
  },
  {
    "id": 2474,
    "start_hour": 711,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2474",
    "gerad_crew_id": "C0187",
    "flight_ids": "LEG_31_44,LEG_31_149"
  },
  {
    "id": 2475,
    "start_hour": 711,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D2475",
    "gerad_crew_id": "C0188",
    "flight_ids": "LEG_31_71,LEG_31_69"
  },
  {
    "id": 2476,
    "start_hour": 711,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D2476",
    "gerad_crew_id": "C0189",
    "flight_ids": "LEG_31_147,LEG_31_205"
  }
];
