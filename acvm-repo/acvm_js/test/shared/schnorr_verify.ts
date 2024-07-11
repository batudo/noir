// See `schnorr_verify_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 85, 211, 103, 78, 2, 81, 24, 70, 225, 193, 6, 216, 123, 47, 216, 123, 239, 136,
  136, 136, 136, 136, 187, 96, 255, 75, 32, 112, 194, 55, 201, 129, 100, 50, 79, 244, 7, 228, 222, 243, 102, 146, 254,
  167, 221, 123, 50, 97, 222, 217, 120, 243, 116, 226, 61, 36, 15, 247, 158, 92, 120, 68, 30, 149, 199, 228, 172, 156,
  147, 243, 242, 184, 60, 33, 79, 202, 83, 242, 180, 60, 35, 207, 202, 115, 242, 188, 188, 32, 47, 202, 75, 242, 178,
  188, 34, 175, 202, 107, 242, 186, 188, 33, 111, 202, 91, 242, 182, 188, 35, 23, 228, 93, 121, 79, 222, 151, 15, 228,
  67, 249, 72, 62, 150, 79, 228, 83, 249, 76, 62, 151, 47, 228, 75, 249, 74, 190, 150, 111, 228, 91, 249, 78, 190, 151,
  31, 228, 71, 249, 73, 126, 150, 95, 228, 87, 185, 40, 191, 201, 37, 249, 93, 46, 203, 31, 114, 69, 254, 148, 171, 97,
  58, 77, 226, 111, 95, 250, 127, 77, 254, 150, 235, 242, 143, 220, 144, 127, 229, 166, 252, 39, 183, 194, 255, 241,
  253, 45, 253, 14, 182, 201, 38, 217, 34, 27, 100, 123, 233, 230, 242, 241, 155, 217, 20, 91, 98, 67, 108, 135, 205,
  176, 21, 54, 194, 54, 216, 4, 91, 96, 3, 180, 79, 243, 180, 78, 227, 180, 77, 211, 180, 76, 195, 180, 75, 179, 133,
  164, 223, 40, 109, 210, 36, 45, 210, 32, 237, 209, 28, 173, 209, 24, 109, 209, 20, 45, 209, 16, 237, 208, 12, 173,
  208, 8, 109, 208, 4, 45, 208, 0, 119, 207, 157, 115, 215, 220, 113, 49, 238, 180, 20, 119, 88, 142, 59, 171, 196, 29,
  85, 227, 46, 106, 113, 246, 245, 56, 235, 70, 156, 109, 51, 206, 50, 61, 179, 244, 220, 18, 157, 231, 192, 167, 11,
  75, 28, 99, 152, 25, 5, 0, 0,
]);

export const initialWitnessMap = new Map([
  [1, '0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a'],
  [2, '0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197'],
  [3, '0x000000000000000000000000000000000000000000000000000000000000002e'],
  [4, '0x00000000000000000000000000000000000000000000000000000000000000ce'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000052'],
  [6, '0x00000000000000000000000000000000000000000000000000000000000000aa'],
  [7, '0x0000000000000000000000000000000000000000000000000000000000000087'],
  [8, '0x000000000000000000000000000000000000000000000000000000000000002a'],
  [9, '0x0000000000000000000000000000000000000000000000000000000000000049'],
  [10, '0x000000000000000000000000000000000000000000000000000000000000009d'],
  [11, '0x0000000000000000000000000000000000000000000000000000000000000050'],
  [12, '0x000000000000000000000000000000000000000000000000000000000000007c'],
  [13, '0x000000000000000000000000000000000000000000000000000000000000009a'],
  [14, '0x00000000000000000000000000000000000000000000000000000000000000aa'],
  [15, '0x00000000000000000000000000000000000000000000000000000000000000df'],
  [16, '0x0000000000000000000000000000000000000000000000000000000000000023'],
  [17, '0x0000000000000000000000000000000000000000000000000000000000000034'],
  [18, '0x0000000000000000000000000000000000000000000000000000000000000010'],
  [19, '0x000000000000000000000000000000000000000000000000000000000000008a'],
  [20, '0x0000000000000000000000000000000000000000000000000000000000000047'],
  [21, '0x0000000000000000000000000000000000000000000000000000000000000063'],
  [22, '0x00000000000000000000000000000000000000000000000000000000000000e8'],
  [23, '0x0000000000000000000000000000000000000000000000000000000000000037'],
  [24, '0x0000000000000000000000000000000000000000000000000000000000000054'],
  [25, '0x0000000000000000000000000000000000000000000000000000000000000096'],
  [26, '0x000000000000000000000000000000000000000000000000000000000000003e'],
  [27, '0x00000000000000000000000000000000000000000000000000000000000000d5'],
  [28, '0x00000000000000000000000000000000000000000000000000000000000000ae'],
  [29, '0x0000000000000000000000000000000000000000000000000000000000000024'],
  [30, '0x000000000000000000000000000000000000000000000000000000000000002d'],
  [31, '0x0000000000000000000000000000000000000000000000000000000000000020'],
  [32, '0x0000000000000000000000000000000000000000000000000000000000000080'],
  [33, '0x000000000000000000000000000000000000000000000000000000000000004d'],
  [34, '0x0000000000000000000000000000000000000000000000000000000000000047'],
  [35, '0x00000000000000000000000000000000000000000000000000000000000000a5'],
  [36, '0x00000000000000000000000000000000000000000000000000000000000000bb'],
  [37, '0x00000000000000000000000000000000000000000000000000000000000000f6'],
  [38, '0x00000000000000000000000000000000000000000000000000000000000000c3'],
  [39, '0x000000000000000000000000000000000000000000000000000000000000000b'],
  [40, '0x000000000000000000000000000000000000000000000000000000000000003b'],
  [41, '0x0000000000000000000000000000000000000000000000000000000000000065'],
  [42, '0x00000000000000000000000000000000000000000000000000000000000000c9'],
  [43, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [44, '0x0000000000000000000000000000000000000000000000000000000000000085'],
  [45, '0x0000000000000000000000000000000000000000000000000000000000000006'],
  [46, '0x000000000000000000000000000000000000000000000000000000000000009e'],
  [47, '0x000000000000000000000000000000000000000000000000000000000000002f'],
  [48, '0x0000000000000000000000000000000000000000000000000000000000000010'],
  [49, '0x00000000000000000000000000000000000000000000000000000000000000e6'],
  [50, '0x0000000000000000000000000000000000000000000000000000000000000030'],
  [51, '0x000000000000000000000000000000000000000000000000000000000000004a'],
  [52, '0x0000000000000000000000000000000000000000000000000000000000000018'],
  [53, '0x000000000000000000000000000000000000000000000000000000000000007c'],
  [54, '0x00000000000000000000000000000000000000000000000000000000000000d0'],
  [55, '0x00000000000000000000000000000000000000000000000000000000000000ab'],
  [56, '0x0000000000000000000000000000000000000000000000000000000000000031'],
  [57, '0x00000000000000000000000000000000000000000000000000000000000000d5'],
  [58, '0x0000000000000000000000000000000000000000000000000000000000000063'],
  [59, '0x0000000000000000000000000000000000000000000000000000000000000084'],
  [60, '0x00000000000000000000000000000000000000000000000000000000000000a3'],
  [61, '0x00000000000000000000000000000000000000000000000000000000000000a6'],
  [62, '0x00000000000000000000000000000000000000000000000000000000000000d5'],
  [63, '0x0000000000000000000000000000000000000000000000000000000000000091'],
  [64, '0x000000000000000000000000000000000000000000000000000000000000000d'],
  [65, '0x000000000000000000000000000000000000000000000000000000000000009c'],
  [66, '0x00000000000000000000000000000000000000000000000000000000000000f9'],
  [67, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [68, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [69, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [70, '0x0000000000000000000000000000000000000000000000000000000000000003'],
  [71, '0x0000000000000000000000000000000000000000000000000000000000000004'],
  [72, '0x0000000000000000000000000000000000000000000000000000000000000005'],
  [73, '0x0000000000000000000000000000000000000000000000000000000000000006'],
  [74, '0x0000000000000000000000000000000000000000000000000000000000000007'],
  [75, '0x0000000000000000000000000000000000000000000000000000000000000008'],
  [76, '0x0000000000000000000000000000000000000000000000000000000000000009'],
]);

export const expectedWitnessMap = new Map(initialWitnessMap).set(
  77,
  '0x0000000000000000000000000000000000000000000000000000000000000001',
);
