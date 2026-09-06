//! Complete enabled kNonEdgeAAPaths fixture table from upstream triangulator_test.cpp at f7c22102.
#![allow(clippy::excessive_precision)]
use nuxie_render_api::RawPath;

pub(super) const NON_EDGE_AA_PATHS: &[fn() -> RawPath] = &[
    // Tests active edges made inactive by splitting.
    // Also tests active edge list forced into an invalid ordering by
    // splitting (mopped up in cleanup_active_edges()).
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(229.127044677734375, 67.34100341796875);
        path.line_to(187.8097381591796875, -6.7729740142822265625);
        path.line_to(171.411407470703125, 50.94266510009765625);
        path.line_to(245.5253753662109375, 9.6253643035888671875);
        path.move_to(208.4683990478515625, 30.284009933471679688);
        path.line_to(171.411407470703125, 50.94266510009765625);
        path.line_to(187.8097381591796875, -6.7729740142822265625);
        path
    },
    // Intersections which fall exactly on the current vertex, and require
    // a restart of the intersection checking.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(314.483551025390625, 486.246002197265625);
        path.line_to(385.41949462890625, 532.8087158203125);
        path.line_to(373.232879638671875, 474.05938720703125);
        path.line_to(326.670166015625, 544.995361328125);
        path.move_to(349.951507568359375, 509.52734375);
        path.line_to(373.232879638671875, 474.05938720703125);
        path.line_to(385.41949462890625, 532.8087158203125);
        path
    },
    // Tests active edges which are removed by splitting.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(343.107391357421875, 613.62176513671875);
        path.line_to(426.632415771484375, 628.5740966796875);
        path.line_to(392.3460693359375, 579.33544921875);
        path.line_to(377.39373779296875, 662.86041259765625);
        path.move_to(384.869873046875, 621.097900390625);
        path.line_to(392.3460693359375, 579.33544921875);
        path.line_to(426.632415771484375, 628.5740966796875);
        path
    },
    // Collinear edges merged in set_top().
    // Also, an intersection between left and right enclosing edges which
    // falls above the current vertex.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(545.95751953125, 791.69854736328125);
        path.line_to(612.05816650390625, 738.494140625);
        path.line_to(552.4056396484375, 732.0460205078125);
        path.line_to(605.61004638671875, 798.14666748046875);
        path.move_to(579.00787353515625, 765.0963134765625);
        path.line_to(552.4056396484375, 732.0460205078125);
        path.line_to(612.05816650390625, 738.494140625);
        path
    },
    // Tests active edges which are made inactive by set_top().
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(819.2725830078125, 751.77447509765625);
        path.line_to(820.70904541015625, 666.933837890625);
        path.line_to(777.57049560546875, 708.63592529296875);
        path.line_to(862.4111328125, 710.0723876953125);
        path.move_to(819.99078369140625, 709.3541259765625);
        path.line_to(777.57049560546875, 708.63592529296875);
        path.line_to(820.70904541015625, 666.933837890625);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(823.33209228515625, 749.052734375);
        path.line_to(823.494873046875, 664.20013427734375);
        path.line_to(780.9871826171875, 706.5450439453125);
        path.line_to(865.8397216796875, 706.70782470703125);
        path.move_to(823.4134521484375, 706.6263427734375);
        path.line_to(780.9871826171875, 706.5450439453125);
        path.line_to(823.494873046875, 664.20013427734375);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(954.862548828125, 562.8349609375);
        path.line_to(899.32818603515625, 498.679443359375);
        path.line_to(895.017578125, 558.52435302734375);
        path.line_to(959.17315673828125, 502.990081787109375);
        path.move_to(927.0953369140625, 530.7572021484375);
        path.line_to(895.017578125, 558.52435302734375);
        path.line_to(899.32818603515625, 498.679443359375);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(958.5330810546875, 547.35516357421875);
        path.line_to(899.93109130859375, 485.989013671875);
        path.line_to(898.54901123046875, 545.97308349609375);
        path.line_to(959.9151611328125, 487.37109375);
        path.move_to(929.2320556640625, 516.67205810546875);
        path.line_to(898.54901123046875, 545.97308349609375);
        path.line_to(899.93109130859375, 485.989013671875);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(389.8609619140625, 369.326873779296875);
        path.line_to(470.6290283203125, 395.33697509765625);
        path.line_to(443.250030517578125, 341.9478759765625);
        path.line_to(417.239959716796875, 422.7159423828125);
        path.move_to(430.244964599609375, 382.3319091796875);
        path.line_to(443.250030517578125, 341.9478759765625);
        path.line_to(470.6290283203125, 395.33697509765625);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(20.0, 20.0);
        path.line_to(50.0, 80.0);
        path.line_to(20.0, 80.0);
        path.move_to(80.0, 50.0);
        path.line_to(50.0, 50.0);
        path.line_to(20.0, 50.0);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(257.19439697265625, 320.876617431640625);
        path.line_to(190.113037109375, 320.58978271484375);
        path.line_to(203.64404296875, 293.8145751953125);
        path.move_to(203.357177734375, 360.896026611328125);
        path.line_to(216.88824462890625, 334.120819091796875);
        path.line_to(230.41925048828125, 307.345611572265625);
        path
    },
    // A degenerate segments case, where both upper and lower segments of
    // a split edge must remain active.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(231.9331207275390625, 306.2012939453125);
        path.line_to(191.4859161376953125, 306.04547119140625);
        path.line_to(231.0659332275390625, 300.2642822265625);
        path.move_to(189.946807861328125, 302.072265625);
        path.line_to(179.79705810546875, 294.859771728515625);
        path.line_to(191.0016021728515625, 296.165679931640625);
        path.move_to(150.8942108154296875, 304.900146484375);
        path.line_to(179.708892822265625, 297.849029541015625);
        path.line_to(190.4742279052734375, 299.11895751953125);
        path
    },
    // Handle the case where edge.dist(edge.fTop) != 0.0.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 400.0);
        path.line_to(138.0, 202.0);
        path.line_to(0.0, 202.0);
        path.move_to(12.62693023681640625, 250.57464599609375);
        path.line_to(8.13896942138671875, 254.556884765625);
        path.line_to(-18.15641021728515625, 220.40203857421875);
        path.line_to(-15.986493110656738281, 219.6513519287109375);
        path.move_to(36.931194305419921875, 282.485504150390625);
        path.line_to(15.617521286010742188, 261.2901611328125);
        path.line_to(10.3829498291015625, 252.565765380859375);
        path.line_to(-16.165292739868164062, 222.646026611328125);
        path
    },
    // A degenerate segments case which exercises inactive edges being
    // made active by splitting.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(690.62127685546875, 509.25555419921875);
        path.line_to(99.336181640625, 511.71405029296875);
        path.line_to(708.362548828125, 512.4349365234375);
        path.line_to(729.9940185546875, 516.3114013671875);
        path.line_to(738.708984375, 518.76995849609375);
        path.line_to(678.3463134765625, 510.0819091796875);
        path.line_to(681.21795654296875, 504.81378173828125);
        path.move_to(758.52764892578125, 521.55963134765625);
        path.line_to(719.1549072265625, 514.50372314453125);
        path.line_to(689.59063720703125, 512.0628662109375);
        path.line_to(679.78216552734375, 507.447845458984375);
        path
    },
    // Tests vertices which become "orphaned" (ie., no connected edges)
    // after simplification.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(217.326019287109375, 166.4752960205078125);
        path.line_to(226.279266357421875, 170.929473876953125);
        path.line_to(234.3973388671875, 177.0623626708984375);
        path.line_to(262.0921630859375, 188.746124267578125);
        path.move_to(196.23638916015625, 174.0722198486328125);
        path.line_to(416.15277099609375, 180.138214111328125);
        path.line_to(192.651947021484375, 304.0228271484375);
        path
    },
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10000.0, 0.0);
        path.line_to(0.0, -1.0);
        path.line_to(10000.0, 0.000001);
        path.line_to(0.0, -30.0);
        path
    },
    // Reduction of Nebraska-StateSeal.svg. Floating point error causes the
    // same edge to be added to more than one poly on the same side.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(170.8199920654296875, 491.86700439453125);
        path.line_to(173.7649993896484375, 489.7340087890625);
        path.line_to(174.1450958251953125, 498.545989990234375);
        path.line_to(171.998992919921875, 500.88201904296875);
        path.move_to(168.2922515869140625, 498.66265869140625);
        path.line_to(169.8589935302734375, 497.94500732421875);
        path.line_to(172.0, 500.88299560546875);
        path.move_to(169.555267333984375, 490.70111083984375);
        path.line_to(173.7649993896484375, 489.7340087890625);
        path.line_to(170.82000732421875, 491.86700439453125);
        path
    },
    // A shape with a vertex collinear to the right hand edge.
    // This messes up find_enclosing_edges.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(80.0, 20.0);
        path.line_to(80.0, 60.0);
        path.line_to(20.0, 60.0);
        path.move_to(80.0, 50.0);
        path.line_to(80.0, 80.0);
        path.line_to(20.0, 80.0);
        path
    },
    // Exercises the case where an edge becomes collinear with *two* of its
    // adjacent neighbour edges after splitting.
    // This is a reduction from
    // http://mooooo.ooo/chebyshev-sine-approximation/horner_ulp.svg
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(351.99298095703125, 348.23046875);
        path.line_to(351.91876220703125, 347.33984375);
        path.line_to(351.91876220703125, 346.1953125);
        path.line_to(351.90313720703125, 347.734375);
        path.line_to(351.90313720703125, 346.1328125);
        path.line_to(351.87579345703125, 347.93359375);
        path.line_to(351.87579345703125, 345.484375);
        path.line_to(351.86407470703125, 347.7890625);
        path.line_to(351.86407470703125, 346.2109375);
        path.line_to(351.84844970703125, 347.63763427734375);
        path.line_to(351.84454345703125, 344.19232177734375);
        path.line_to(351.78204345703125, 346.9483642578125);
        path.line_to(351.758636474609375, 347.18310546875);
        path.line_to(351.75469970703125, 346.75);
        path.line_to(351.75469970703125, 345.46875);
        path.line_to(352.5546875, 345.46875);
        path.line_to(352.55078125, 347.01953125);
        path.line_to(351.75079345703125, 347.02313232421875);
        path.line_to(351.74688720703125, 346.15203857421875);
        path.line_to(351.74688720703125, 347.646148681640625);
        path.line_to(352.5390625, 346.94140625);
        path.line_to(351.73907470703125, 346.94268798828125);
        path.line_to(351.73516845703125, 344.48565673828125);
        path.line_to(352.484375, 346.73828125);
        path.line_to(351.68438720703125, 346.7401123046875);
        path.line_to(352.4765625, 346.546875);
        path.line_to(351.67657470703125, 346.54937744140625);
        path.line_to(352.47265625, 346.75390625);
        path.line_to(351.67266845703125, 346.756622314453125);
        path.line_to(351.66876220703125, 345.612091064453125);
        path
    },
    // A path which contains out-of-range colinear intersections.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 63.39080047607421875);
        path.line_to(-0.70804601907730102539, 63.14350128173828125);
        path.line_to(-7.8608899287380243391e-17, 64.14080047607421875);
        path.move_to(0.0, 64.14080047607421875);
        path.line_to(44.285900115966796875, 64.14080047607421875);
        path.line_to(0.0, 62.64080047607421875);
        path.move_to(21.434900283813476562, -0.24732701480388641357);
        path.line_to(-0.70804601907730102539, 63.14350128173828125);
        path.line_to(0.70804601907730102539, 63.6381988525390625);
        path
    },
    // Disabled upstream conic/quad fixtures remain excluded.

    // Disabled upstream conic/quad fixtures remain excluded.

    // A path which hangs during simplification. It produces an edge which is
    // to the left of its own endpoints, which causes an infinite loop in the
    // right-enclosing-edge splitting.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.75001740455627441406, 23.051967620849609375);
        path.line_to(5.8471612930297851562, 22.731662750244140625);
        path.line_to(10.749670028686523438, 22.253145217895507812);
        path.line_to(13.115868568420410156, 22.180681228637695312);
        path.line_to(15.418928146362304688, 22.340015411376953125);
        path.line_to(17.654022216796875, 22.82159423828125);
        path.line_to(19.81632232666015625, 23.715869903564453125);
        path.line_to(40.0, 0.0);
        path.line_to(5.5635203441547955577e-15, 0.0);
        path.line_to(5.5635203441547955577e-15, 47.0);
        path.line_to(-1.4210854715202003717e-14, 21.713298797607421875);
        path.line_to(0.75001740455627441406, 21.694292068481445312);
        path.line_to(0.75001740455627441406, 23.051967620849609375);
        path
    },
    // Reduction from skbug.com/7911 that causes a crash due to splitting a
    // zombie edge.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(0.0, 1.0927740941146660348e+24);
        path.line_to(2.9333931225865729333e+32, 16476101.0);
        path.line_to(1.0927731573659435417e+24, 1.0927740941146660348e+24);
        path.line_to(1.0927740941146660348e+24, 3.7616281094287041715e-37);
        path.line_to(1.0927740941146660348e+24, 1.0927740941146660348e+24);
        path.line_to(1.3061803026169399536e-33, 1.0927740941146660348e+24);
        path.line_to(4.7195362919941370727e-16, -8.4247545146051822591e+32);
        path
    },
    // From crbug.com/844873. Crashes trying to merge a zombie edge.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(316.000579833984375, -4338355948977389568.0);
        path.line_to(1.5069369808623501312e+20, 75180972320904708096.0);
        path.line_to(1.5069369808623501312e+20, 75180972320904708096.0);
        path.line_to(771.21014404296875, -4338355948977389568.0);
        path.line_to(316.000579833984375, -4338355948977389568.0);
        path.move_to(354.208984375, -4338355948977389568.0);
        path.line_to(773.00177001953125, -4338355948977389568.0);
        path.line_to(1.5069369808623501312e+20, 75180972320904708096.0);
        path.line_to(1.5069369808623501312e+20, 75180972320904708096.0);
        path.line_to(354.208984375, -4338355948977389568.0);
        path
    },
    // From crbug.com/844873. Hangs repeatedly splitting alternate vertices.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(10.0, -1e+20);
        path.line_to(11.0, 25000.0);
        path.line_to(10.0, 25000.0);
        path.line_to(11.0, 25010.0);
        path
    },
    // Reduction from circular_arcs_stroke_and_fill_round GM which
    // repeatedly splits on the opposite edge from case 34 above.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(16.25, 26.495191574096679688);
        path.line_to(32.420825958251953125, 37.377376556396484375);
        path.line_to(25.176382064819335938, 39.31851959228515625);
        path.move_to(20.0, 20.0);
        path.line_to(28.847436904907226562, 37.940830230712890625);
        path.line_to(25.17638397216796875, 39.31851959228515625);
        path
    },
    // Reduction from crbug.com/843135 where an intersection is found
    // below the bottom of both intersected edges.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(-2791476679359332352.0, 2608107002026524672.0);
        path.line_to(0.0, 11.95427703857421875);
        path.line_to(-2781824066779086848.0, 2599088532777598976.0);
        path.line_to(-7772.6875, 7274.0);
        path
    },
    // Reduction from crbug.com/843135. Exercises a case where an intersection
    // is missed.
    // This causes bad ordering in the active edge list.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(-1.0662557646016024569e+23, 9.9621425197286319718e+22);
        path.line_to(-121806400.0, 113805032.0);
        path.line_to(-120098872.0, 112209680.0);
        path.line_to(6.2832999862817380468e-36, 2.9885697364807128906);
        path
    },
    // Reduction from crbug.com/851409. Exercises collinear last vertex.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(2072553216.0, 0.0);
        path.line_to(2072553216.0, 1.0);
        path.line_to(2072553472.0, -13.5);
        path.line_to(2072553216.0, 0.0);
        path.line_to(2072553472.0, -6.5);
        path
    },
    // Another reduction from crbug.com/851409. Exercises two sequential
    // collinear edges.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(2072553216.0, 0.0);
        path.line_to(2072553216.0, 1.0);
        path.line_to(2072553472.0, -13.0);
        path.line_to(2072553216.0, 0.0);
        path.line_to(2072553472.0, -6.0);
        path.line_to(2072553472.0, -13.0);
        path
    },
    // Reduction from crbug.com/860655. Cause is three collinear edges
    // discovered during
    // sanitize_contours pass, before the vertices have been found coincident.
    || -> RawPath {
        let mut path = RawPath::new();
        path.move_to(32572426382475264.0, -3053391034974208.0);
        path.line_to(521289856.0, -48865776.0);
        path.line_to(130322464.0, -12215873.0);
        path.move_to(32572426382475264.0, -3053391034974208.0);
        path.line_to(521289856.0, -48865776.0);
        path.line_to(130322464.0, -12215873.0);
        path.move_to(32572426382475264.0, -3053391034974208.0);
        path.line_to(32114477642022912.0, -3010462031544320.0);
        path.line_to(32111784697528320.0, -3010209702215680.0);
        path
    },
];
