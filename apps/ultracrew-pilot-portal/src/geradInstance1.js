// AUTO-GENERATED — do not edit by hand.
// Source: benchmarks/gerad-g2014-22/instance1/crew.csv + duties.csv
// Generator: scripts/gen_all_gerad_js.py
// Pipeline: GERAD instance1 → Roster/Duty/CrewMember → workers[]/shifts[]
//
// GERAD G-2014-22 Instance 1 (Kasirzadeh, Saddoune & Soumis 2014)
// 33 crew · 385 duties · 759h horizon
// Normalization offset: 11h subtracted from all start_hour values.

export const GERAD_INSTANCE1_META = {
  "source": "GERAD G-2014-22 Instance 1 (Kasirzadeh, Saddoune & Soumis 2014)",
  "total_crew": 33,
  "total_duties": 385,
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
  "horizon_hours": 759,
  "max_hours_per_worker": 80,
  "normalization_offset_hours": 11,
  "note": "start_hour values are normalized (min subtracted) so the optimizer receives relative hours rather than absolute epoch offsets. Temporal structure (gaps, overlaps, rest periods) is preserved."
};

// workers[]: each GERAD CrewMember projected to UltraCrew Worker schema.
// id: numeric crew_id (C0001 → 1), skills: [qualification]
export const GERAD_INSTANCE1_WORKERS = [
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
    "base": "BASE2",
    "gerad_id": "C0008",
    "contract_type": "part_time"
  },
  {
    "id": 9,
    "skills": [
      "A320"
    ],
    "name": "Pierre Faure",
    "base": "BASE2",
    "gerad_id": "C0009",
    "contract_type": "full_time"
  },
  {
    "id": 10,
    "skills": [
      "A319"
    ],
    "name": "Margot Gauthier",
    "base": "BASE2",
    "gerad_id": "C0010",
    "contract_type": "full_time"
  },
  {
    "id": 11,
    "skills": [
      "A319"
    ],
    "name": "Anais Clement",
    "base": "BASE2",
    "gerad_id": "C0011",
    "contract_type": "full_time"
  },
  {
    "id": 12,
    "skills": [
      "A321"
    ],
    "name": "Nicolas Richard",
    "base": "BASE2",
    "gerad_id": "C0012",
    "contract_type": "full_time"
  },
  {
    "id": 13,
    "skills": [
      "A319"
    ],
    "name": "Stephane Dupont",
    "base": "BASE2",
    "gerad_id": "C0013",
    "contract_type": "part_time"
  },
  {
    "id": 14,
    "skills": [
      "A320"
    ],
    "name": "Baptiste Robert",
    "base": "BASE2",
    "gerad_id": "C0014",
    "contract_type": "full_time"
  },
  {
    "id": 15,
    "skills": [
      "A319"
    ],
    "name": "Quentin Garcia",
    "base": "BASE2",
    "gerad_id": "C0015",
    "contract_type": "full_time"
  },
  {
    "id": 16,
    "skills": [
      "A320"
    ],
    "name": "Florian David",
    "base": "BASE2",
    "gerad_id": "C0016",
    "contract_type": "full_time"
  },
  {
    "id": 17,
    "skills": [
      "A320"
    ],
    "name": "Celine Petit",
    "base": "BASE2",
    "gerad_id": "C0017",
    "contract_type": "full_time"
  },
  {
    "id": 18,
    "skills": [
      "A321"
    ],
    "name": "Sebastien Thomas",
    "base": "BASE2",
    "gerad_id": "C0018",
    "contract_type": "full_time"
  },
  {
    "id": 19,
    "skills": [
      "A319"
    ],
    "name": "Camille Simon",
    "base": "BASE2",
    "gerad_id": "C0019",
    "contract_type": "full_time"
  },
  {
    "id": 20,
    "skills": [
      "A321"
    ],
    "name": "Baptiste Andre",
    "base": "BASE2",
    "gerad_id": "C0020",
    "contract_type": "full_time"
  },
  {
    "id": 21,
    "skills": [
      "A320"
    ],
    "name": "Sophie Andre",
    "base": "BASE2",
    "gerad_id": "C0021",
    "contract_type": "full_time"
  },
  {
    "id": 22,
    "skills": [
      "A320"
    ],
    "name": "Laure Legrand",
    "base": "BASE2",
    "gerad_id": "C0022",
    "contract_type": "full_time"
  },
  {
    "id": 23,
    "skills": [
      "A319"
    ],
    "name": "Clara Francois",
    "base": "BASE2",
    "gerad_id": "C0023",
    "contract_type": "full_time"
  },
  {
    "id": 24,
    "skills": [
      "A321"
    ],
    "name": "Clara Bernard",
    "base": "BASE2",
    "gerad_id": "C0024",
    "contract_type": "full_time"
  },
  {
    "id": 25,
    "skills": [
      "A321"
    ],
    "name": "Benoit Garnier",
    "base": "BASE2",
    "gerad_id": "C0025",
    "contract_type": "part_time"
  },
  {
    "id": 26,
    "skills": [
      "A319"
    ],
    "name": "Margot Clement",
    "base": "BASE2",
    "gerad_id": "C0026",
    "contract_type": "full_time"
  },
  {
    "id": 27,
    "skills": [
      "A320"
    ],
    "name": "Elise Bonnet",
    "base": "BASE2",
    "gerad_id": "C0027",
    "contract_type": "full_time"
  },
  {
    "id": 28,
    "skills": [
      "A319"
    ],
    "name": "Laurent Gauthier",
    "base": "BASE3",
    "gerad_id": "C0028",
    "contract_type": "full_time"
  },
  {
    "id": 29,
    "skills": [
      "A321"
    ],
    "name": "Margot Morin",
    "base": "BASE3",
    "gerad_id": "C0029",
    "contract_type": "full_time"
  },
  {
    "id": 30,
    "skills": [
      "A319"
    ],
    "name": "Thibault Richard",
    "base": "BASE3",
    "gerad_id": "C0030",
    "contract_type": "full_time"
  },
  {
    "id": 31,
    "skills": [
      "A319"
    ],
    "name": "Sandrine Leroy",
    "base": "BASE3",
    "gerad_id": "C0031",
    "contract_type": "full_time"
  },
  {
    "id": 32,
    "skills": [
      "A319"
    ],
    "name": "Benoit Faure",
    "base": "BASE3",
    "gerad_id": "C0032",
    "contract_type": "full_time"
  },
  {
    "id": 33,
    "skills": [
      "A320"
    ],
    "name": "Michel Lopez",
    "base": "BASE3",
    "gerad_id": "C0033",
    "contract_type": "full_time"
  }
];

// shifts[]: each GERAD Duty projected to UltraCrew Shift schema.
// id: numeric duty_id, start_hour: normalized FDP report time,
// duration_hours: FDP length (release - report), required_skill: crew qualification.
export const GERAD_INSTANCE1_SHIFTS = [
  {
    "id": 1,
    "start_hour": 679,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0001",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_29_1"
  },
  {
    "id": 2,
    "start_hour": 684,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0002",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_30_11,LEG_30_0,LEG_30_22,LEG_30_23,LEG_30_4"
  },
  {
    "id": 3,
    "start_hour": 681,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0003",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_29_3"
  },
  {
    "id": 4,
    "start_hour": 684,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0004",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_30_19,LEG_30_21,LEG_30_5"
  },
  {
    "id": 5,
    "start_hour": 676,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0005",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_29_2,LEG_29_29,LEG_29_28"
  },
  {
    "id": 6,
    "start_hour": 684,
    "duration_hours": 2,
    "required_skill": "A319",
    "gerad_duty_id": "D0006",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_30_26"
  },
  {
    "id": 7,
    "start_hour": 682,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0007",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_29_8"
  },
  {
    "id": 8,
    "start_hour": 685,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0008",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_30_9"
  },
  {
    "id": 9,
    "start_hour": 674,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0009",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_29_15,LEG_29_17"
  },
  {
    "id": 10,
    "start_hour": 663,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0010",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_29_27,LEG_29_14"
  },
  {
    "id": 11,
    "start_hour": 506,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0011",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_22_20,LEG_22_21"
  },
  {
    "id": 12,
    "start_hour": 1,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0012",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_01_30,LEG_01_28"
  },
  {
    "id": 13,
    "start_hour": 10,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0013",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_01_14"
  },
  {
    "id": 14,
    "start_hour": 12,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0014",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_02_35"
  },
  {
    "id": 15,
    "start_hour": 4,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0015",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_01_31,LEG_01_2,LEG_01_3,LEG_01_33,LEG_01_5"
  },
  {
    "id": 16,
    "start_hour": 25,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0016",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_02_7,LEG_02_6,LEG_02_8"
  },
  {
    "id": 17,
    "start_hour": 3,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0017",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_01_6,LEG_01_8,LEG_01_9"
  },
  {
    "id": 18,
    "start_hour": 22,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0018",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_02_32,LEG_02_30,LEG_02_28"
  },
  {
    "id": 19,
    "start_hour": 6,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0019",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_01_21,LEG_01_4,LEG_01_32"
  },
  {
    "id": 20,
    "start_hour": 25,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0020",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_02_10,LEG_02_11,LEG_02_23"
  },
  {
    "id": 21,
    "start_hour": 50,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0021",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_03_22,LEG_03_14"
  },
  {
    "id": 22,
    "start_hour": 60,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0022",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_04_35"
  },
  {
    "id": 23,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0023",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_04_18,LEG_04_15,LEG_04_9"
  },
  {
    "id": 24,
    "start_hour": 94,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0024",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_05_28,LEG_05_26,LEG_05_25"
  },
  {
    "id": 25,
    "start_hour": 124,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0025",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_06_28,LEG_06_2,LEG_06_3,LEG_06_31,LEG_06_30"
  },
  {
    "id": 26,
    "start_hour": 145,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0026",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_07_10,LEG_07_11,LEG_07_19"
  },
  {
    "id": 27,
    "start_hour": 383,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0027",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_17_3,LEG_17_2"
  },
  {
    "id": 28,
    "start_hour": 373,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0028",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_17_17,LEG_17_26"
  },
  {
    "id": 29,
    "start_hour": 388,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0029",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_17_30,LEG_17_0,LEG_17_1,LEG_17_32,LEG_17_5"
  },
  {
    "id": 30,
    "start_hour": 409,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0030",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_18_8,LEG_18_7,LEG_18_9"
  },
  {
    "id": 31,
    "start_hour": 394,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0031",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_17_10"
  },
  {
    "id": 32,
    "start_hour": 396,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0032",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_18_33"
  },
  {
    "id": 33,
    "start_hour": 385,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0033",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_17_27,LEG_17_28,LEG_17_23,LEG_17_18,LEG_17_19"
  },
  {
    "id": 34,
    "start_hour": 411,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0034",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_18_14,LEG_18_13,LEG_18_22"
  },
  {
    "id": 35,
    "start_hour": 434,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0035",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_19_16,LEG_19_11,LEG_19_22,LEG_19_13"
  },
  {
    "id": 36,
    "start_hour": 457,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0036",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_20_2,LEG_20_14"
  },
  {
    "id": 37,
    "start_hour": 335,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0037",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_15_3,LEG_15_2,LEG_15_30,LEG_15_0"
  },
  {
    "id": 38,
    "start_hour": 325,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0038",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_15_17,LEG_15_26"
  },
  {
    "id": 39,
    "start_hour": 346,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0039",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_15_10"
  },
  {
    "id": 40,
    "start_hour": 348,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0040",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_16_33"
  },
  {
    "id": 41,
    "start_hour": 342,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0041",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_15_29,LEG_15_4,LEG_15_5"
  },
  {
    "id": 42,
    "start_hour": 361,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0042",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_16_8,LEG_16_7,LEG_16_9"
  },
  {
    "id": 43,
    "start_hour": 344,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0043",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_15_1,LEG_15_32,LEG_15_31"
  },
  {
    "id": 44,
    "start_hour": 361,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0044",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_16_11,LEG_16_12"
  },
  {
    "id": 45,
    "start_hour": 386,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0045",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_17_24,LEG_17_29,LEG_17_4"
  },
  {
    "id": 46,
    "start_hour": 145,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0046",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_07_30,LEG_07_28"
  },
  {
    "id": 47,
    "start_hour": 153,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0047",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_07_29,LEG_07_24"
  },
  {
    "id": 48,
    "start_hour": 157,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0048",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_08_25,LEG_08_27,LEG_08_21,LEG_08_4,LEG_08_5"
  },
  {
    "id": 49,
    "start_hour": 193,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0049",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_09_7,LEG_09_6,LEG_09_8"
  },
  {
    "id": 50,
    "start_hour": 152,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0050",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_07_3,LEG_07_34,LEG_07_33"
  },
  {
    "id": 51,
    "start_hour": 169,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0051",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_08_10,LEG_08_11,LEG_08_19"
  },
  {
    "id": 52,
    "start_hour": 194,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0052",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_09_18,LEG_09_15,LEG_09_9"
  },
  {
    "id": 53,
    "start_hour": 214,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0053",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_10_32,LEG_10_30,LEG_10_28"
  },
  {
    "id": 54,
    "start_hour": 443,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0054",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_19_21"
  },
  {
    "id": 55,
    "start_hour": 458,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0055",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_20_22,LEG_20_21,LEG_20_15,LEG_20_16"
  },
  {
    "id": 56,
    "start_hour": 483,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0056",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_21_13,LEG_21_12,LEG_21_21"
  },
  {
    "id": 57,
    "start_hour": 299,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0057",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_13_20"
  },
  {
    "id": 58,
    "start_hour": 314,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0058",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_14_23,LEG_14_30,LEG_14_25"
  },
  {
    "id": 59,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0059",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_15_27,LEG_15_28,LEG_15_23,LEG_15_18,LEG_15_19"
  },
  {
    "id": 60,
    "start_hour": 363,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0060",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_16_14,LEG_16_13,LEG_16_22"
  },
  {
    "id": 61,
    "start_hour": 627,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0061",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_27_22,LEG_27_9"
  },
  {
    "id": 62,
    "start_hour": 639,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0062",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_28_28,LEG_28_18,LEG_28_26"
  },
  {
    "id": 63,
    "start_hour": 663,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0063",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_29_25,LEG_29_6,LEG_29_7,LEG_29_13"
  },
  {
    "id": 64,
    "start_hour": 322,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0064",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_14_15"
  },
  {
    "id": 65,
    "start_hour": 324,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0065",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_15_33"
  },
  {
    "id": 66,
    "start_hour": 320,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0066",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_14_5,LEG_14_35,LEG_14_34"
  },
  {
    "id": 67,
    "start_hour": 337,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0067",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_15_11,LEG_15_21,LEG_15_25"
  },
  {
    "id": 68,
    "start_hour": 362,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0068",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_16_24,LEG_16_29,LEG_16_4,LEG_16_5"
  },
  {
    "id": 69,
    "start_hour": 385,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0069",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_17_8,LEG_17_7"
  },
  {
    "id": 70,
    "start_hour": 318,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0070",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_14_27,LEG_14_17,LEG_14_18"
  },
  {
    "id": 71,
    "start_hour": 339,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0071",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_15_14,LEG_15_13,LEG_15_22"
  },
  {
    "id": 72,
    "start_hour": 362,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0072",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_16_20,LEG_16_15,LEG_16_16,LEG_16_6"
  },
  {
    "id": 73,
    "start_hour": 551,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0073",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_24_3,LEG_24_2,LEG_24_30,LEG_24_0"
  },
  {
    "id": 74,
    "start_hour": 562,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0074",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_24_10"
  },
  {
    "id": 75,
    "start_hour": 564,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0075",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_25_33"
  },
  {
    "id": 76,
    "start_hour": 541,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0076",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_24_17,LEG_24_26"
  },
  {
    "id": 77,
    "start_hour": 558,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0077",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_24_29,LEG_24_4,LEG_24_5"
  },
  {
    "id": 78,
    "start_hour": 577,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0078",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_25_8,LEG_25_7,LEG_25_9"
  },
  {
    "id": 79,
    "start_hour": 560,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0079",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_24_1,LEG_24_32,LEG_24_31"
  },
  {
    "id": 80,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0080",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_25_11,LEG_25_21,LEG_25_25"
  },
  {
    "id": 81,
    "start_hour": 606,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0081",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_26_19,LEG_26_14"
  },
  {
    "id": 82,
    "start_hour": 602,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0082",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_26_16,LEG_26_11,LEG_26_22,LEG_26_13"
  },
  {
    "id": 83,
    "start_hour": 629,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0083",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_27_10,LEG_27_17"
  },
  {
    "id": 84,
    "start_hour": 239,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0084",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_11_3,LEG_11_2,LEG_11_31,LEG_11_0"
  },
  {
    "id": 85,
    "start_hour": 250,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0085",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_11_14"
  },
  {
    "id": 86,
    "start_hour": 252,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0086",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_12_30"
  },
  {
    "id": 87,
    "start_hour": 249,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0087",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_11_29,LEG_11_24"
  },
  {
    "id": 88,
    "start_hour": 248,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0088",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_11_1,LEG_11_34,LEG_11_33"
  },
  {
    "id": 89,
    "start_hour": 265,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0089",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_12_7,LEG_12_8,LEG_12_16"
  },
  {
    "id": 90,
    "start_hour": 294,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0090",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_13_18,LEG_13_4,LEG_13_5"
  },
  {
    "id": 91,
    "start_hour": 313,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0091",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_14_9,LEG_14_8,LEG_14_10"
  },
  {
    "id": 92,
    "start_hour": 246,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0092",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_11_26,LEG_11_16,LEG_11_17"
  },
  {
    "id": 93,
    "start_hour": 267,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0093",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_12_10,LEG_12_9,LEG_12_17"
  },
  {
    "id": 94,
    "start_hour": 290,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0094",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_13_15,LEG_13_11,LEG_13_6"
  },
  {
    "id": 95,
    "start_hour": 310,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0095",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_14_33,LEG_14_31,LEG_14_29"
  },
  {
    "id": 96,
    "start_hour": 124,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0096",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_06_8,LEG_06_16,LEG_06_20"
  },
  {
    "id": 97,
    "start_hour": 146,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0097",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_07_22,LEG_07_14"
  },
  {
    "id": 98,
    "start_hour": 156,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0098",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_08_35,LEG_08_31,LEG_08_0,LEG_08_29,LEG_08_24"
  },
  {
    "id": 99,
    "start_hour": 181,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0099",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_09_25,LEG_09_27,LEG_09_26,LEG_09_16"
  },
  {
    "id": 100,
    "start_hour": 359,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0100",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_16_3,LEG_16_2,LEG_16_30,LEG_16_0"
  },
  {
    "id": 101,
    "start_hour": 349,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0101",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_16_17,LEG_16_26"
  },
  {
    "id": 102,
    "start_hour": 370,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0102",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_16_10"
  },
  {
    "id": 103,
    "start_hour": 372,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0103",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_17_33"
  },
  {
    "id": 104,
    "start_hour": 368,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0104",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_16_1,LEG_16_32,LEG_16_31"
  },
  {
    "id": 105,
    "start_hour": 385,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0105",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_17_11,LEG_17_12,LEG_17_21"
  },
  {
    "id": 106,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0106",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_18_20,LEG_18_15,LEG_18_5"
  },
  {
    "id": 107,
    "start_hour": 433,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0107",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_19_4,LEG_19_3,LEG_19_5"
  },
  {
    "id": 108,
    "start_hour": 580,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0108",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_25_12"
  },
  {
    "id": 109,
    "start_hour": 602,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0109",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_26_20"
  },
  {
    "id": 110,
    "start_hour": 460,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0110",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_20_7,LEG_20_19,LEG_20_23"
  },
  {
    "id": 111,
    "start_hour": 482,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0111",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_21_23,LEG_21_22,LEG_21_17"
  },
  {
    "id": 112,
    "start_hour": 218,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0112",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_10_18,LEG_10_19"
  },
  {
    "id": 113,
    "start_hour": 144,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0113",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_07_0,LEG_07_1,LEG_07_31,LEG_07_2,LEG_07_9"
  },
  {
    "id": 114,
    "start_hour": 166,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0114",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_08_32,LEG_08_30,LEG_08_28"
  },
  {
    "id": 115,
    "start_hour": 191,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0115",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_09_3,LEG_09_2"
  },
  {
    "id": 116,
    "start_hour": 154,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0116",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_07_17"
  },
  {
    "id": 117,
    "start_hour": 171,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0117",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_08_13,LEG_08_12,LEG_08_20"
  },
  {
    "id": 118,
    "start_hour": 200,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0118",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_09_19,LEG_09_23"
  },
  {
    "id": 119,
    "start_hour": 218,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0119",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_10_22,LEG_10_26,LEG_10_16"
  },
  {
    "id": 120,
    "start_hour": 657,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0120",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_28_3"
  },
  {
    "id": 121,
    "start_hour": 660,
    "duration_hours": 22,
    "required_skill": "A320",
    "gerad_duty_id": "D0121",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_29_19,LEG_29_21,LEG_29_5"
  },
  {
    "id": 122,
    "start_hour": 658,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0122",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_28_8"
  },
  {
    "id": 123,
    "start_hour": 661,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0123",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_29_9"
  },
  {
    "id": 124,
    "start_hour": 421,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0124",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_19_12,LEG_19_23"
  },
  {
    "id": 125,
    "start_hour": 433,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0125",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_19_26,LEG_19_25"
  },
  {
    "id": 126,
    "start_hour": 431,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0126",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_19_2,LEG_19_1,LEG_19_27,LEG_19_0"
  },
  {
    "id": 127,
    "start_hour": 441,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0127",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_19_24,LEG_19_10"
  },
  {
    "id": 128,
    "start_hour": 445,
    "duration_hours": 23,
    "required_skill": "A320",
    "gerad_duty_id": "D0128",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_20_12,LEG_20_1,LEG_20_17,LEG_20_3"
  },
  {
    "id": 129,
    "start_hour": 478,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0129",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_21_30,LEG_21_7,LEG_21_8"
  },
  {
    "id": 130,
    "start_hour": 438,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0130",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_19_19,LEG_19_14,LEG_19_15"
  },
  {
    "id": 131,
    "start_hour": 459,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0131",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_20_9,LEG_20_8,LEG_20_20"
  },
  {
    "id": 132,
    "start_hour": 482,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0132",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_21_19,LEG_21_14,LEG_21_15,LEG_21_6"
  },
  {
    "id": 133,
    "start_hour": 469,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0133",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_21_16,LEG_21_25"
  },
  {
    "id": 134,
    "start_hour": 490,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0134",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_21_9"
  },
  {
    "id": 135,
    "start_hour": 492,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0135",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_22_33"
  },
  {
    "id": 136,
    "start_hour": 486,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0136",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_21_28,LEG_21_4,LEG_21_5"
  },
  {
    "id": 137,
    "start_hour": 505,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0137",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_22_8,LEG_22_7,LEG_22_9"
  },
  {
    "id": 138,
    "start_hour": 488,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0138",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_21_3,LEG_21_32,LEG_21_31"
  },
  {
    "id": 139,
    "start_hour": 505,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0139",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_22_11,LEG_22_12,LEG_22_25"
  },
  {
    "id": 140,
    "start_hour": 530,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0140",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_23_24,LEG_23_29,LEG_23_4,LEG_23_5"
  },
  {
    "id": 141,
    "start_hour": 553,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0141",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_24_8,LEG_24_7,LEG_24_9"
  },
  {
    "id": 142,
    "start_hour": 575,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0142",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_25_3,LEG_25_2,LEG_25_30,LEG_25_0"
  },
  {
    "id": 143,
    "start_hour": 586,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0143",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_25_10"
  },
  {
    "id": 144,
    "start_hour": 588,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0144",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_26_29"
  },
  {
    "id": 145,
    "start_hour": 565,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0145",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_25_17,LEG_25_26"
  },
  {
    "id": 146,
    "start_hour": 584,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0146",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_25_1,LEG_25_32,LEG_25_31"
  },
  {
    "id": 147,
    "start_hour": 601,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0147",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_26_7"
  },
  {
    "id": 148,
    "start_hour": 616,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0148",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_27_0,LEG_27_12"
  },
  {
    "id": 149,
    "start_hour": 202,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0149",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_09_14"
  },
  {
    "id": 150,
    "start_hour": 204,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0150",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_10_35"
  },
  {
    "id": 151,
    "start_hour": 196,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0151",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_09_31,LEG_09_0,LEG_09_1,LEG_09_34,LEG_09_5"
  },
  {
    "id": 152,
    "start_hour": 217,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0152",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_10_7,LEG_10_6,LEG_10_8"
  },
  {
    "id": 153,
    "start_hour": 201,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0153",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_09_29,LEG_09_24"
  },
  {
    "id": 154,
    "start_hour": 205,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0154",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_10_25,LEG_10_27,LEG_10_21,LEG_10_4,LEG_10_5"
  },
  {
    "id": 155,
    "start_hour": 241,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0155",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_11_7,LEG_11_6,LEG_11_8"
  },
  {
    "id": 156,
    "start_hour": 198,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0156",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_09_21,LEG_09_4,LEG_09_33"
  },
  {
    "id": 157,
    "start_hour": 217,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0157",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_10_10,LEG_10_11,LEG_10_23"
  },
  {
    "id": 158,
    "start_hour": 242,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0158",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_11_22,LEG_11_9"
  },
  {
    "id": 159,
    "start_hour": 262,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0159",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_12_28,LEG_12_26,LEG_12_25"
  },
  {
    "id": 160,
    "start_hour": 179,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0160",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_08_23"
  },
  {
    "id": 161,
    "start_hour": 194,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0161",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_09_22,LEG_09_17"
  },
  {
    "id": 162,
    "start_hour": 219,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0162",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_10_13,LEG_10_12,LEG_10_20"
  },
  {
    "id": 163,
    "start_hour": 242,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0163",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_11_18,LEG_11_15,LEG_11_5"
  },
  {
    "id": 164,
    "start_hour": 265,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0164",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_12_5,LEG_12_27,LEG_12_0,LEG_12_29"
  },
  {
    "id": 165,
    "start_hour": 289,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0165",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_13_7,LEG_13_8,LEG_13_16"
  },
  {
    "id": 166,
    "start_hour": 251,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0166",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_11_23"
  },
  {
    "id": 167,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0167",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_12_19,LEG_12_23,LEG_12_13,LEG_12_14"
  },
  {
    "id": 168,
    "start_hour": 291,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0168",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_13_10,LEG_13_14"
  },
  {
    "id": 169,
    "start_hour": 315,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0169",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_14_14,LEG_14_13,LEG_14_21"
  },
  {
    "id": 170,
    "start_hour": 493,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0170",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_22_17,LEG_22_26"
  },
  {
    "id": 171,
    "start_hour": 503,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0171",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_22_3,LEG_22_2,LEG_22_30,LEG_22_0"
  },
  {
    "id": 172,
    "start_hour": 514,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0172",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_22_10"
  },
  {
    "id": 173,
    "start_hour": 516,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0173",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_23_33"
  },
  {
    "id": 174,
    "start_hour": 505,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0174",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_22_27,LEG_22_28,LEG_22_1,LEG_22_32,LEG_22_5"
  },
  {
    "id": 175,
    "start_hour": 529,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0175",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_23_8,LEG_23_7,LEG_23_9"
  },
  {
    "id": 176,
    "start_hour": 510,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0176",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_22_29,LEG_22_4,LEG_22_31"
  },
  {
    "id": 177,
    "start_hour": 529,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0177",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_23_11,LEG_23_12,LEG_23_21"
  },
  {
    "id": 178,
    "start_hour": 557,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0178",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_24_15"
  },
  {
    "id": 179,
    "start_hour": 554,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0179",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_24_20,LEG_24_21"
  },
  {
    "id": 180,
    "start_hour": 50,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0180",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_03_18,LEG_03_19"
  },
  {
    "id": 181,
    "start_hour": 708,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0181",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_30_20"
  },
  {
    "id": 182,
    "start_hour": 724,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0182",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_31_18"
  },
  {
    "id": 183,
    "start_hour": 631,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0183",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_27_8,LEG_27_23"
  },
  {
    "id": 184,
    "start_hour": 637,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0184",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_28_11,LEG_28_27,LEG_28_6,LEG_28_7,LEG_28_22"
  },
  {
    "id": 185,
    "start_hour": 676,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0185",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_29_18,LEG_29_20"
  },
  {
    "id": 186,
    "start_hour": 700,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0186",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_30_18,LEG_30_12,LEG_30_10"
  },
  {
    "id": 187,
    "start_hour": 626,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0187",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_27_18,LEG_27_19"
  },
  {
    "id": 188,
    "start_hour": 634,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0188",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_27_11"
  },
  {
    "id": 189,
    "start_hour": 637,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0189",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_28_9"
  },
  {
    "id": 190,
    "start_hour": 624,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0190",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_27_5,LEG_27_4,LEG_27_26"
  },
  {
    "id": 191,
    "start_hour": 648,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0191",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_28_16,LEG_28_17,LEG_28_19"
  },
  {
    "id": 192,
    "start_hour": 628,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0192",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_27_25,LEG_27_24,LEG_27_15"
  },
  {
    "id": 193,
    "start_hour": 649,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0193",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_28_0,LEG_28_24,LEG_28_25,LEG_28_4"
  },
  {
    "id": 194,
    "start_hour": 631,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0194",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_27_6"
  },
  {
    "id": 195,
    "start_hour": 649,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0195",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_28_13,LEG_28_2,LEG_28_30"
  },
  {
    "id": 196,
    "start_hour": 73,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0196",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_04_30,LEG_04_28"
  },
  {
    "id": 197,
    "start_hour": 71,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0197",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_04_3,LEG_04_2,LEG_04_31,LEG_04_0"
  },
  {
    "id": 198,
    "start_hour": 80,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0198",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_04_1,LEG_04_34,LEG_04_33"
  },
  {
    "id": 199,
    "start_hour": 97,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0199",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_05_7"
  },
  {
    "id": 200,
    "start_hour": 121,
    "duration_hours": 6,
    "required_skill": "A320",
    "gerad_duty_id": "D0200",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_06_1,LEG_06_12"
  },
  {
    "id": 201,
    "start_hour": 81,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0201",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_04_29,LEG_04_24"
  },
  {
    "id": 202,
    "start_hour": 85,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0202",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_05_22,LEG_05_24,LEG_05_23,LEG_05_13,LEG_05_14"
  },
  {
    "id": 203,
    "start_hour": 123,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0203",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_06_10,LEG_06_9,LEG_06_17"
  },
  {
    "id": 204,
    "start_hour": 146,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0204",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_07_18,LEG_07_15"
  },
  {
    "id": 205,
    "start_hour": 23,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0205",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_02_3,LEG_02_2,LEG_02_31,LEG_02_0"
  },
  {
    "id": 206,
    "start_hour": 13,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0206",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_02_25,LEG_02_27,LEG_02_21,LEG_02_4,LEG_02_5"
  },
  {
    "id": 207,
    "start_hour": 49,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0207",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_03_7,LEG_03_6,LEG_03_8"
  },
  {
    "id": 208,
    "start_hour": 33,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0208",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_02_29,LEG_02_24"
  },
  {
    "id": 209,
    "start_hour": 32,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0209",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_02_1,LEG_02_34,LEG_02_33"
  },
  {
    "id": 210,
    "start_hour": 49,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0210",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_03_10,LEG_03_11,LEG_03_23"
  },
  {
    "id": 211,
    "start_hour": 74,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0211",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_04_22,LEG_04_14"
  },
  {
    "id": 212,
    "start_hour": 84,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0212",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_05_30"
  },
  {
    "id": 213,
    "start_hour": 386,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0213",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_17_20,LEG_17_15,LEG_17_16,LEG_17_6"
  },
  {
    "id": 214,
    "start_hour": 412,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0214",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_18_30,LEG_18_0,LEG_18_1,LEG_18_32,LEG_18_31"
  },
  {
    "id": 215,
    "start_hour": 433,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0215",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_19_6,LEG_19_7,LEG_19_17"
  },
  {
    "id": 216,
    "start_hour": 395,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0216",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_17_25"
  },
  {
    "id": 217,
    "start_hour": 410,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0217",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_18_24,LEG_18_23,LEG_18_18,LEG_18_19"
  },
  {
    "id": 218,
    "start_hour": 435,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0218",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_19_9,LEG_19_8,LEG_19_18"
  },
  {
    "id": 219,
    "start_hour": 226,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0219",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_10_17"
  },
  {
    "id": 220,
    "start_hour": 243,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0220",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_11_13,LEG_11_12,LEG_11_20"
  },
  {
    "id": 221,
    "start_hour": 266,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0221",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_12_15,LEG_12_11,LEG_12_21,LEG_12_12"
  },
  {
    "id": 222,
    "start_hour": 724,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0222",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_31_2,LEG_31_29"
  },
  {
    "id": 223,
    "start_hour": 711,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0223",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_31_27,LEG_31_14"
  },
  {
    "id": 224,
    "start_hour": 722,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0224",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_31_15,LEG_31_17"
  },
  {
    "id": 225,
    "start_hour": 58,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0225",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_03_17"
  },
  {
    "id": 226,
    "start_hour": 75,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0226",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_04_13,LEG_04_12,LEG_04_20"
  },
  {
    "id": 227,
    "start_hour": 98,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0227",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_05_15,LEG_05_11,LEG_05_21,LEG_05_12"
  },
  {
    "id": 228,
    "start_hour": 517,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0228",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_23_17,LEG_23_26"
  },
  {
    "id": 229,
    "start_hour": 527,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0229",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_23_3,LEG_23_2,LEG_23_30,LEG_23_0"
  },
  {
    "id": 230,
    "start_hour": 538,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0230",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_23_10"
  },
  {
    "id": 231,
    "start_hour": 540,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0231",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_24_33"
  },
  {
    "id": 232,
    "start_hour": 536,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0232",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_23_1,LEG_23_32,LEG_23_31"
  },
  {
    "id": 233,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0233",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_24_11,LEG_24_12,LEG_24_25"
  },
  {
    "id": 234,
    "start_hour": 578,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0234",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_25_24,LEG_25_29,LEG_25_4,LEG_25_5"
  },
  {
    "id": 235,
    "start_hour": 601,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0235",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_26_5,LEG_26_4,LEG_26_6"
  },
  {
    "id": 236,
    "start_hour": 529,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0236",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_23_27,LEG_23_28,LEG_23_23,LEG_23_18,LEG_23_19"
  },
  {
    "id": 237,
    "start_hour": 555,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0237",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_24_14,LEG_24_13,LEG_24_22"
  },
  {
    "id": 238,
    "start_hour": 578,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0238",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_25_20,LEG_25_15,LEG_25_16,LEG_25_6"
  },
  {
    "id": 239,
    "start_hour": 457,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0239",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_20_26,LEG_20_25"
  },
  {
    "id": 240,
    "start_hour": 462,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0240",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_20_13,LEG_20_0"
  },
  {
    "id": 241,
    "start_hour": 480,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0241",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_21_0,LEG_21_1,LEG_21_29,LEG_21_2"
  },
  {
    "id": 242,
    "start_hour": 462,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0242",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_20_24,LEG_20_4,LEG_20_27"
  },
  {
    "id": 243,
    "start_hour": 481,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0243",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_21_10,LEG_21_11"
  },
  {
    "id": 244,
    "start_hour": 509,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0244",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_22_15,LEG_22_16,LEG_22_6"
  },
  {
    "id": 245,
    "start_hour": 599,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0245",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_26_2,LEG_26_1,LEG_26_27,LEG_26_0"
  },
  {
    "id": 246,
    "start_hour": 589,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0246",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_26_12,LEG_26_23"
  },
  {
    "id": 247,
    "start_hour": 606,
    "duration_hours": 5,
    "required_skill": "A321",
    "gerad_duty_id": "D0247",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_26_26,LEG_26_3"
  },
  {
    "id": 248,
    "start_hour": 613,
    "duration_hours": 23,
    "required_skill": "A321",
    "gerad_duty_id": "D0248",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_27_1,LEG_27_2,LEG_27_7"
  },
  {
    "id": 249,
    "start_hour": 636,
    "duration_hours": 24,
    "required_skill": "A321",
    "gerad_duty_id": "D0249",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_28_21,LEG_28_23,LEG_28_5,LEG_28_29"
  },
  {
    "id": 250,
    "start_hour": 660,
    "duration_hours": 2,
    "required_skill": "A321",
    "gerad_duty_id": "D0250",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_29_26"
  },
  {
    "id": 251,
    "start_hour": 601,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0251",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_26_24,LEG_26_25,LEG_26_28"
  },
  {
    "id": 252,
    "start_hour": 626,
    "duration_hours": 13,
    "required_skill": "A320",
    "gerad_duty_id": "D0252",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_27_14,LEG_27_21"
  },
  {
    "id": 253,
    "start_hour": 652,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0253",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_28_20,LEG_28_14,LEG_28_10"
  },
  {
    "id": 254,
    "start_hour": 412,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0254",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_18_12,LEG_18_21,LEG_18_25"
  },
  {
    "id": 255,
    "start_hour": 434,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0255",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_19_20,LEG_19_28"
  },
  {
    "id": 256,
    "start_hour": 457,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0256",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_20_6"
  },
  {
    "id": 257,
    "start_hour": 76,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0257",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_04_11,LEG_04_19,LEG_04_23"
  },
  {
    "id": 258,
    "start_hour": 98,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0258",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_05_19,LEG_05_29"
  },
  {
    "id": 259,
    "start_hour": 121,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0259",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_06_7"
  },
  {
    "id": 260,
    "start_hour": 82,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0260",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_04_17"
  },
  {
    "id": 261,
    "start_hour": 99,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0261",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_05_10,LEG_05_9,LEG_05_17"
  },
  {
    "id": 262,
    "start_hour": 122,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0262",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_06_15,LEG_06_11,LEG_06_6"
  },
  {
    "id": 263,
    "start_hour": 142,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0263",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_07_32,LEG_07_26,LEG_07_16"
  },
  {
    "id": 264,
    "start_hour": 488,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0264",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_21_20,LEG_21_24"
  },
  {
    "id": 265,
    "start_hour": 506,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0265",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_22_24,LEG_22_23,LEG_22_18,LEG_22_19"
  },
  {
    "id": 266,
    "start_hour": 531,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0266",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_23_14,LEG_23_13,LEG_23_22"
  },
  {
    "id": 267,
    "start_hour": 289,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0267",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_13_27,LEG_13_25"
  },
  {
    "id": 268,
    "start_hour": 277,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0268",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_13_22,LEG_13_24,LEG_13_29,LEG_13_0"
  },
  {
    "id": 269,
    "start_hour": 312,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0269",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_14_0,LEG_14_1,LEG_14_32,LEG_14_4"
  },
  {
    "id": 270,
    "start_hour": 297,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0270",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_13_26,LEG_13_21"
  },
  {
    "id": 271,
    "start_hour": 301,
    "duration_hours": 25,
    "required_skill": "A321",
    "gerad_duty_id": "D0271",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_14_26,LEG_14_28,LEG_14_22,LEG_14_6,LEG_14_7"
  },
  {
    "id": 272,
    "start_hour": 337,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0272",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_15_8,LEG_15_7,LEG_15_9"
  },
  {
    "id": 273,
    "start_hour": 292,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0273",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_13_28,LEG_13_2,LEG_13_30"
  },
  {
    "id": 274,
    "start_hour": 313,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0274",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_14_11,LEG_14_12,LEG_14_20"
  },
  {
    "id": 275,
    "start_hour": 338,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0275",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_15_20,LEG_15_15"
  },
  {
    "id": 276,
    "start_hour": 47,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0276",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_03_3,LEG_03_2"
  },
  {
    "id": 277,
    "start_hour": 49,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0277",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_03_30,LEG_03_28"
  },
  {
    "id": 278,
    "start_hour": 37,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0278",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_03_25,LEG_03_27,LEG_03_21,LEG_03_4,LEG_03_5"
  },
  {
    "id": 279,
    "start_hour": 73,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0279",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_04_7,LEG_04_6,LEG_04_8"
  },
  {
    "id": 280,
    "start_hour": 57,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0280",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_03_29,LEG_03_24"
  },
  {
    "id": 281,
    "start_hour": 61,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0281",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_04_25,LEG_04_27,LEG_04_21,LEG_04_4,LEG_04_5"
  },
  {
    "id": 282,
    "start_hour": 97,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0282",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_05_5,LEG_05_4,LEG_05_6"
  },
  {
    "id": 283,
    "start_hour": 155,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0283",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_07_23"
  },
  {
    "id": 284,
    "start_hour": 170,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0284",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_08_22,LEG_08_26,LEG_08_16,LEG_08_17"
  },
  {
    "id": 285,
    "start_hour": 195,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0285",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_09_13,LEG_09_12,LEG_09_20"
  },
  {
    "id": 286,
    "start_hour": 263,
    "duration_hours": 6,
    "required_skill": "A319",
    "gerad_duty_id": "D0286",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_12_2,LEG_12_1"
  },
  {
    "id": 287,
    "start_hour": 253,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0287",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_12_22,LEG_12_24,LEG_12_18,LEG_12_3"
  },
  {
    "id": 288,
    "start_hour": 267,
    "duration_hours": 7,
    "required_skill": "A321",
    "gerad_duty_id": "D0288",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_12_4,LEG_12_6"
  },
  {
    "id": 289,
    "start_hour": 705,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0289",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_30_3"
  },
  {
    "id": 290,
    "start_hour": 708,
    "duration_hours": 26,
    "required_skill": "A319",
    "gerad_duty_id": "D0290",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_31_19,LEG_31_12,LEG_31_10"
  },
  {
    "id": 291,
    "start_hour": 700,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0291",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_30_2,LEG_30_29,LEG_30_28"
  },
  {
    "id": 292,
    "start_hour": 708,
    "duration_hours": 2,
    "required_skill": "A320",
    "gerad_duty_id": "D0292",
    "gerad_crew_id": "C0014",
    "flight_ids": "LEG_31_26"
  },
  {
    "id": 293,
    "start_hour": 703,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0293",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_30_1"
  },
  {
    "id": 294,
    "start_hour": 708,
    "duration_hours": 21,
    "required_skill": "A319",
    "gerad_duty_id": "D0294",
    "gerad_crew_id": "C0015",
    "flight_ids": "LEG_31_11,LEG_31_0,LEG_31_22,LEG_31_23,LEG_31_4"
  },
  {
    "id": 295,
    "start_hour": 706,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0295",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_30_8"
  },
  {
    "id": 296,
    "start_hour": 709,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0296",
    "gerad_crew_id": "C0016",
    "flight_ids": "LEG_31_9"
  },
  {
    "id": 297,
    "start_hour": 687,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0297",
    "gerad_crew_id": "C0017",
    "flight_ids": "LEG_30_27,LEG_30_14"
  },
  {
    "id": 298,
    "start_hour": 215,
    "duration_hours": 9,
    "required_skill": "A321",
    "gerad_duty_id": "D0298",
    "gerad_crew_id": "C0018",
    "flight_ids": "LEG_10_3,LEG_10_2,LEG_10_31,LEG_10_0"
  },
  {
    "id": 299,
    "start_hour": 225,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0299",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_10_29,LEG_10_24"
  },
  {
    "id": 300,
    "start_hour": 229,
    "duration_hours": 22,
    "required_skill": "A319",
    "gerad_duty_id": "D0300",
    "gerad_crew_id": "C0019",
    "flight_ids": "LEG_11_25,LEG_11_27,LEG_11_21,LEG_11_4"
  },
  {
    "id": 301,
    "start_hour": 226,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0301",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_10_14"
  },
  {
    "id": 302,
    "start_hour": 228,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0302",
    "gerad_crew_id": "C0020",
    "flight_ids": "LEG_11_35"
  },
  {
    "id": 303,
    "start_hour": 224,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0303",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_10_1,LEG_10_34,LEG_10_33"
  },
  {
    "id": 304,
    "start_hour": 241,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0304",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_11_10,LEG_11_11,LEG_11_19"
  },
  {
    "id": 305,
    "start_hour": 275,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0305",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_12_20"
  },
  {
    "id": 306,
    "start_hour": 290,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0306",
    "gerad_crew_id": "C0021",
    "flight_ids": "LEG_13_19,LEG_13_3,LEG_13_31"
  },
  {
    "id": 307,
    "start_hour": 32,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0307",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_02_19"
  },
  {
    "id": 308,
    "start_hour": 340,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0308",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_15_12"
  },
  {
    "id": 309,
    "start_hour": 368,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0309",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_16_21,LEG_16_25"
  },
  {
    "id": 310,
    "start_hour": 390,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0310",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_17_9,LEG_17_31"
  },
  {
    "id": 311,
    "start_hour": 409,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0311",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_18_11"
  },
  {
    "id": 312,
    "start_hour": 121,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0312",
    "gerad_crew_id": "C0022",
    "flight_ids": "LEG_06_27,LEG_06_25"
  },
  {
    "id": 313,
    "start_hour": 129,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0313",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_06_26,LEG_06_21"
  },
  {
    "id": 314,
    "start_hour": 133,
    "duration_hours": 25,
    "required_skill": "A319",
    "gerad_duty_id": "D0314",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_07_25,LEG_07_27,LEG_07_21,LEG_07_4,LEG_07_5"
  },
  {
    "id": 315,
    "start_hour": 169,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0315",
    "gerad_crew_id": "C0023",
    "flight_ids": "LEG_08_7,LEG_08_6,LEG_08_8"
  },
  {
    "id": 316,
    "start_hour": 126,
    "duration_hours": 8,
    "required_skill": "A321",
    "gerad_duty_id": "D0316",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_06_23,LEG_06_13,LEG_06_14"
  },
  {
    "id": 317,
    "start_hour": 147,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0317",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_07_13,LEG_07_12,LEG_07_20"
  },
  {
    "id": 318,
    "start_hour": 170,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0318",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_08_18,LEG_08_15,LEG_08_9"
  },
  {
    "id": 319,
    "start_hour": 190,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0319",
    "gerad_crew_id": "C0024",
    "flight_ids": "LEG_09_32,LEG_09_30,LEG_09_28"
  },
  {
    "id": 320,
    "start_hour": 2,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0320",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_01_18,LEG_01_15,LEG_01_29,LEG_01_25"
  },
  {
    "id": 321,
    "start_hour": 30,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0321",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_02_26,LEG_02_16,LEG_02_17"
  },
  {
    "id": 322,
    "start_hour": 51,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0322",
    "gerad_crew_id": "C0028",
    "flight_ids": "LEG_03_13,LEG_03_12,LEG_03_20"
  },
  {
    "id": 323,
    "start_hour": 610,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0323",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_26_15"
  },
  {
    "id": 324,
    "start_hour": 613,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0324",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_27_3"
  },
  {
    "id": 325,
    "start_hour": 636,
    "duration_hours": 4,
    "required_skill": "A321",
    "gerad_duty_id": "D0325",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_28_12"
  },
  {
    "id": 326,
    "start_hour": 604,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0326",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_26_8,LEG_26_17,LEG_26_21"
  },
  {
    "id": 327,
    "start_hour": 628,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0327",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_27_20,LEG_27_16,LEG_27_13"
  },
  {
    "id": 328,
    "start_hour": 655,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0328",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_28_1"
  },
  {
    "id": 329,
    "start_hour": 660,
    "duration_hours": 4,
    "required_skill": "A320",
    "gerad_duty_id": "D0329",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_29_11"
  },
  {
    "id": 330,
    "start_hour": 0,
    "duration_hours": 10,
    "required_skill": "A319",
    "gerad_duty_id": "D0330",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_01_0,LEG_01_1,LEG_01_26,LEG_01_16"
  },
  {
    "id": 331,
    "start_hour": 7,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0331",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_01_12,LEG_01_20"
  },
  {
    "id": 332,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0332",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_02_18,LEG_02_15,LEG_02_9"
  },
  {
    "id": 333,
    "start_hour": 46,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0333",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_03_32,LEG_03_26,LEG_03_16"
  },
  {
    "id": 334,
    "start_hour": 4,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0334",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_01_11,LEG_01_19,LEG_01_23"
  },
  {
    "id": 335,
    "start_hour": 26,
    "duration_hours": 10,
    "required_skill": "A320",
    "gerad_duty_id": "D0335",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_02_22,LEG_02_14"
  },
  {
    "id": 336,
    "start_hour": 36,
    "duration_hours": 28,
    "required_skill": "A320",
    "gerad_duty_id": "D0336",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_03_35,LEG_03_31,LEG_03_0,LEG_03_1,LEG_03_34,LEG_03_33"
  },
  {
    "id": 337,
    "start_hour": 73,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0337",
    "gerad_crew_id": "C0002",
    "flight_ids": "LEG_04_10"
  },
  {
    "id": 338,
    "start_hour": 10,
    "duration_hours": 4,
    "required_skill": "A319",
    "gerad_duty_id": "D0338",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_01_17"
  },
  {
    "id": 339,
    "start_hour": 27,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0339",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_02_13,LEG_02_12,LEG_02_20"
  },
  {
    "id": 340,
    "start_hour": 53,
    "duration_hours": 7,
    "required_skill": "A319",
    "gerad_duty_id": "D0340",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_03_15,LEG_03_9"
  },
  {
    "id": 341,
    "start_hour": 70,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0341",
    "gerad_crew_id": "C0003",
    "flight_ids": "LEG_04_32,LEG_04_26,LEG_04_16"
  },
  {
    "id": 342,
    "start_hour": 100,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0342",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_05_8,LEG_05_16,LEG_05_20"
  },
  {
    "id": 343,
    "start_hour": 122,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0343",
    "gerad_crew_id": "C0004",
    "flight_ids": "LEG_06_19,LEG_06_29,LEG_06_0"
  },
  {
    "id": 344,
    "start_hour": 458,
    "duration_hours": 11,
    "required_skill": "A321",
    "gerad_duty_id": "D0344",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_20_18,LEG_20_10,LEG_20_11,LEG_20_5"
  },
  {
    "id": 345,
    "start_hour": 481,
    "duration_hours": 13,
    "required_skill": "A321",
    "gerad_duty_id": "D0345",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_21_26,LEG_21_27,LEG_21_18"
  },
  {
    "id": 346,
    "start_hour": 507,
    "duration_hours": 12,
    "required_skill": "A321",
    "gerad_duty_id": "D0346",
    "gerad_crew_id": "C0029",
    "flight_ids": "LEG_22_14,LEG_22_13,LEG_22_22"
  },
  {
    "id": 347,
    "start_hour": 167,
    "duration_hours": 6,
    "required_skill": "A321",
    "gerad_duty_id": "D0347",
    "gerad_crew_id": "C0025",
    "flight_ids": "LEG_08_3,LEG_08_2"
  },
  {
    "id": 348,
    "start_hour": 178,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0348",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_08_14"
  },
  {
    "id": 349,
    "start_hour": 180,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0349",
    "gerad_crew_id": "C0026",
    "flight_ids": "LEG_09_35"
  },
  {
    "id": 350,
    "start_hour": 176,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0350",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_08_1,LEG_08_34,LEG_08_33"
  },
  {
    "id": 351,
    "start_hour": 193,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0351",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_09_10,LEG_09_11"
  },
  {
    "id": 352,
    "start_hour": 221,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0352",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_10_15,LEG_10_9"
  },
  {
    "id": 353,
    "start_hour": 238,
    "duration_hours": 11,
    "required_skill": "A320",
    "gerad_duty_id": "D0353",
    "gerad_crew_id": "C0027",
    "flight_ids": "LEG_11_32,LEG_11_30,LEG_11_28"
  },
  {
    "id": 354,
    "start_hour": 289,
    "duration_hours": 10,
    "required_skill": "A321",
    "gerad_duty_id": "D0354",
    "gerad_crew_id": "C0005",
    "flight_ids": "LEG_13_1,LEG_13_12,LEG_13_23,LEG_13_13"
  },
  {
    "id": 355,
    "start_hour": 295,
    "duration_hours": 8,
    "required_skill": "A320",
    "gerad_duty_id": "D0355",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_13_9,LEG_13_17"
  },
  {
    "id": 356,
    "start_hour": 314,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0356",
    "gerad_crew_id": "C0006",
    "flight_ids": "LEG_14_19,LEG_14_16,LEG_14_3,LEG_14_2"
  },
  {
    "id": 357,
    "start_hour": 95,
    "duration_hours": 9,
    "required_skill": "A319",
    "gerad_duty_id": "D0357",
    "gerad_crew_id": "C0008",
    "flight_ids": "LEG_05_2,LEG_05_1,LEG_05_27,LEG_05_0"
  },
  {
    "id": 358,
    "start_hour": 102,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0358",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_05_18,LEG_05_3"
  },
  {
    "id": 359,
    "start_hour": 109,
    "duration_hours": 25,
    "required_skill": "A320",
    "gerad_duty_id": "D0359",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_06_22,LEG_06_24,LEG_06_18,LEG_06_4,LEG_06_5"
  },
  {
    "id": 360,
    "start_hour": 145,
    "duration_hours": 9,
    "required_skill": "A320",
    "gerad_duty_id": "D0360",
    "gerad_crew_id": "C0009",
    "flight_ids": "LEG_07_7,LEG_07_6,LEG_07_8"
  },
  {
    "id": 361,
    "start_hour": 407,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0361",
    "gerad_crew_id": "C0010",
    "flight_ids": "LEG_18_3,LEG_18_2,LEG_18_29,LEG_18_4"
  },
  {
    "id": 362,
    "start_hour": 397,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0362",
    "gerad_crew_id": "C0011",
    "flight_ids": "LEG_18_17,LEG_18_26"
  },
  {
    "id": 363,
    "start_hour": 418,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0363",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_18_10"
  },
  {
    "id": 364,
    "start_hour": 420,
    "duration_hours": 3,
    "required_skill": "A321",
    "gerad_duty_id": "D0364",
    "gerad_crew_id": "C0012",
    "flight_ids": "LEG_19_29"
  },
  {
    "id": 365,
    "start_hour": 409,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0365",
    "gerad_crew_id": "C0013",
    "flight_ids": "LEG_18_27,LEG_18_28,LEG_18_16,LEG_18_6"
  },
  {
    "id": 366,
    "start_hour": 323,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0366",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_14_24"
  },
  {
    "id": 367,
    "start_hour": 338,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0367",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_15_24,LEG_15_16,LEG_15_6"
  },
  {
    "id": 368,
    "start_hour": 361,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0368",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_16_27,LEG_16_28,LEG_16_23,LEG_16_18,LEG_16_19"
  },
  {
    "id": 369,
    "start_hour": 387,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0369",
    "gerad_crew_id": "C0030",
    "flight_ids": "LEG_17_14,LEG_17_13,LEG_17_22"
  },
  {
    "id": 370,
    "start_hour": 530,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0370",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_23_20,LEG_23_15,LEG_23_16,LEG_23_6"
  },
  {
    "id": 371,
    "start_hour": 553,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0371",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_24_27,LEG_24_28,LEG_24_23,LEG_24_18,LEG_24_19"
  },
  {
    "id": 372,
    "start_hour": 579,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0372",
    "gerad_crew_id": "C0031",
    "flight_ids": "LEG_25_14,LEG_25_13,LEG_25_22"
  },
  {
    "id": 373,
    "start_hour": 539,
    "duration_hours": 3,
    "required_skill": "A319",
    "gerad_duty_id": "D0373",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_23_25"
  },
  {
    "id": 374,
    "start_hour": 554,
    "duration_hours": 11,
    "required_skill": "A319",
    "gerad_duty_id": "D0374",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_24_24,LEG_24_16,LEG_24_6"
  },
  {
    "id": 375,
    "start_hour": 577,
    "duration_hours": 13,
    "required_skill": "A319",
    "gerad_duty_id": "D0375",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_25_27,LEG_25_28,LEG_25_23,LEG_25_18,LEG_25_19"
  },
  {
    "id": 376,
    "start_hour": 603,
    "duration_hours": 12,
    "required_skill": "A319",
    "gerad_duty_id": "D0376",
    "gerad_crew_id": "C0032",
    "flight_ids": "LEG_26_10,LEG_26_9,LEG_26_18"
  },
  {
    "id": 377,
    "start_hour": 679,
    "duration_hours": 7,
    "required_skill": "A320",
    "gerad_duty_id": "D0377",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_29_12,LEG_29_10"
  },
  {
    "id": 378,
    "start_hour": 706,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0378",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_30_16,LEG_30_24"
  },
  {
    "id": 379,
    "start_hour": 711,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0379",
    "gerad_crew_id": "C0033",
    "flight_ids": "LEG_31_25,LEG_31_6,LEG_31_7,LEG_31_13"
  },
  {
    "id": 380,
    "start_hour": 673,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0380",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_29_0,LEG_29_22,LEG_29_23,LEG_29_4"
  },
  {
    "id": 381,
    "start_hour": 698,
    "duration_hours": 8,
    "required_skill": "A319",
    "gerad_duty_id": "D0381",
    "gerad_crew_id": "C0007",
    "flight_ids": "LEG_30_15,LEG_30_17"
  },
  {
    "id": 382,
    "start_hour": 657,
    "duration_hours": 3,
    "required_skill": "A320",
    "gerad_duty_id": "D0382",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_28_15"
  },
  {
    "id": 383,
    "start_hour": 682,
    "duration_hours": 5,
    "required_skill": "A320",
    "gerad_duty_id": "D0383",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_29_16,LEG_29_24"
  },
  {
    "id": 384,
    "start_hour": 687,
    "duration_hours": 21,
    "required_skill": "A320",
    "gerad_duty_id": "D0384",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_30_25,LEG_30_6,LEG_30_7,LEG_30_13"
  },
  {
    "id": 385,
    "start_hour": 723,
    "duration_hours": 12,
    "required_skill": "A320",
    "gerad_duty_id": "D0385",
    "gerad_crew_id": "C0001",
    "flight_ids": "LEG_31_21,LEG_31_5,LEG_31_16,LEG_31_24"
  }
];
