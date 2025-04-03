; ModuleID = 'autocfg_92ffc430b5b2e973_1.85648e23cc8d862b-cgu.0'
source_filename = "autocfg_92ffc430b5b2e973_1.85648e23cc8d862b-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@alloc_f93507f8ba4b5780b14b2c2584609be0 = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\F0?" }>, align 8
@alloc_ef0a1f828f3393ef691f2705e817091c = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\00@" }>, align 8

; core::f64::<impl f64>::total_cmp
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17ha43bff13dd5cb4c1E"(ptr align 8 %self, ptr align 8 %other) unnamed_addr #0 !dbg !17 {
start:
  %other.dbg.spill6 = alloca [8 x i8], align 8
  %self.dbg.spill5 = alloca [8 x i8], align 8
  %self.dbg.spill4 = alloca [8 x i8], align 8
  %self.dbg.spill2 = alloca [8 x i8], align 8
  %other.dbg.spill = alloca [8 x i8], align 8
  %self.dbg.spill = alloca [8 x i8], align 8
  %right = alloca [8 x i8], align 8
  %left = alloca [8 x i8], align 8
  store ptr %self, ptr %self.dbg.spill, align 8
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill, metadata !27, metadata !DIExpression()), !dbg !36
  store ptr %other, ptr %other.dbg.spill, align 8
  call void @llvm.dbg.declare(metadata ptr %other.dbg.spill, metadata !28, metadata !DIExpression()), !dbg !36
  call void @llvm.dbg.declare(metadata ptr %left, metadata !29, metadata !DIExpression()), !dbg !37
  call void @llvm.dbg.declare(metadata ptr %right, metadata !33, metadata !DIExpression()), !dbg !38
  %self1 = load double, ptr %self, align 8, !dbg !39
  store double %self1, ptr %self.dbg.spill2, align 8, !dbg !39
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill2, metadata !40, metadata !DIExpression()), !dbg !50
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill2, metadata !51, metadata !DIExpression()), !dbg !58
  %_4 = bitcast double %self1 to i64, !dbg !60
  store i64 %_4, ptr %left, align 8, !dbg !39
  %self3 = load double, ptr %other, align 8, !dbg !61
  store double %self3, ptr %self.dbg.spill4, align 8, !dbg !61
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill4, metadata !48, metadata !DIExpression()), !dbg !62
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill4, metadata !56, metadata !DIExpression()), !dbg !63
  %_7 = bitcast double %self3 to i64, !dbg !65
  store i64 %_7, ptr %right, align 8, !dbg !61
  %_13 = load i64, ptr %left, align 8, !dbg !66
  %_12 = ashr i64 %_13, 63, !dbg !66
  %_10 = lshr i64 %_12, 1, !dbg !66
  %0 = load i64, ptr %left, align 8, !dbg !66
  %1 = xor i64 %0, %_10, !dbg !66
  store i64 %1, ptr %left, align 8, !dbg !66
  %_18 = load i64, ptr %right, align 8, !dbg !67
  %_17 = ashr i64 %_18, 63, !dbg !67
  %_15 = lshr i64 %_17, 1, !dbg !67
  %2 = load i64, ptr %right, align 8, !dbg !67
  %3 = xor i64 %2, %_15, !dbg !67
  store i64 %3, ptr %right, align 8, !dbg !67
  store ptr %left, ptr %self.dbg.spill5, align 8, !dbg !68
  call void @llvm.dbg.declare(metadata ptr %self.dbg.spill5, metadata !69, metadata !DIExpression()), !dbg !80
  store ptr %right, ptr %other.dbg.spill6, align 8, !dbg !68
  call void @llvm.dbg.declare(metadata ptr %other.dbg.spill6, metadata !79, metadata !DIExpression()), !dbg !80
  %_21 = load i64, ptr %left, align 8, !dbg !81
  %_22 = load i64, ptr %right, align 8, !dbg !81
  %4 = icmp sgt i64 %_21, %_22, !dbg !81
  %5 = zext i1 %4 to i8, !dbg !81
  %6 = icmp slt i64 %_21, %_22, !dbg !81
  %7 = zext i1 %6 to i8, !dbg !81
  %_0 = sub nsw i8 %5, %7, !dbg !81
  ret i8 %_0, !dbg !82
}

; autocfg_92ffc430b5b2e973_1::probe
; Function Attrs: uwtable
define void @_ZN26autocfg_92ffc430b5b2e973_15probe17h9dc646ddc0469464E() unnamed_addr #1 !dbg !83 {
start:
; call core::f64::<impl f64>::total_cmp
  %_1 = call i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17ha43bff13dd5cb4c1E"(ptr align 8 @alloc_f93507f8ba4b5780b14b2c2584609be0, ptr align 8 @alloc_ef0a1f828f3393ef691f2705e817091c), !dbg !88
  ret void, !dbg !89
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #2

attributes #0 = { inlinehint uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #1 = { uwtable "target-cpu"="x86-64" "target-features"="+cx16,+sse3,+sahf" }
attributes #2 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}
!llvm.dbg.cu = !{!4}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"CodeView", i32 1}
!2 = !{i32 2, !"Debug Info Version", i32 3}
!3 = !{!"rustc version 1.81.0 (eeb90cda1 2024-09-04)"}
!4 = distinct !DICompileUnit(language: DW_LANG_Rust, file: !5, producer: "clang LLVM (rustc version 1.81.0 (eeb90cda1 2024-09-04))", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, enums: !6, splitDebugInlining: false)
!5 = !DIFile(filename: "autocfg_92ffc430b5b2e973_1\\@\\autocfg_92ffc430b5b2e973_1.85648e23cc8d862b-cgu.0", directory: "C:\\Users\\jorda\\.cargo\\registry\\src\\index.crates.io-6f17d22bba15001f\\num-traits-0.2.19")
!6 = !{!7}
!7 = !DICompositeType(tag: DW_TAG_enumeration_type, name: "Ordering", scope: !9, file: !8, baseType: !11, size: 8, align: 8, flags: DIFlagEnumClass, elements: !13)
!8 = !DIFile(filename: "<unknown>", directory: "")
!9 = !DINamespace(name: "cmp", scope: !10)
!10 = !DINamespace(name: "core", scope: null)
!11 = !DIDerivedType(tag: DW_TAG_typedef, name: "i8", file: !8, baseType: !12)
!12 = !DIBasicType(name: "__int8", size: 8, encoding: DW_ATE_signed)
!13 = !{!14, !15, !16}
!14 = !DIEnumerator(name: "Less", value: -1)
!15 = !DIEnumerator(name: "Equal", value: 0)
!16 = !DIEnumerator(name: "Greater", value: 1)
!17 = distinct !DISubprogram(name: "total_cmp", linkageName: "_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17ha43bff13dd5cb4c1E", scope: !19, file: !18, line: 1461, type: !21, scopeLine: 1461, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !4, templateParams: !35, retainedNodes: !26)
!18 = !DIFile(filename: "/rustc/eeb90cda1969383f56a2637cbd3037bdf598841c\\library\\core\\src\\num\\f64.rs", directory: "", checksumkind: CSK_SHA256, checksum: "a91d80d417a6fe2e70e07f573ff847923926533e18dcc1b70e519d619c670c01")
!19 = !DINamespace(name: "impl$0", scope: !20)
!20 = !DINamespace(name: "f64", scope: !10)
!21 = !DISubroutineType(types: !22)
!22 = !{!7, !23, !23}
!23 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "ref$<f64>", baseType: !24, size: 64, align: 64, dwarfAddressSpace: 0)
!24 = !DIDerivedType(tag: DW_TAG_typedef, name: "f64", file: !8, baseType: !25)
!25 = !DIBasicType(name: "double", size: 64, encoding: DW_ATE_float)
!26 = !{!27, !28, !29, !33}
!27 = !DILocalVariable(name: "self", arg: 1, scope: !17, file: !18, line: 1461, type: !23)
!28 = !DILocalVariable(name: "other", arg: 2, scope: !17, file: !18, line: 1461, type: !23)
!29 = !DILocalVariable(name: "left", scope: !30, file: !18, line: 1462, type: !31, align: 8)
!30 = distinct !DILexicalBlock(scope: !17, file: !18, line: 1462)
!31 = !DIDerivedType(tag: DW_TAG_typedef, name: "i64", file: !8, baseType: !32)
!32 = !DIBasicType(name: "__int64", size: 64, encoding: DW_ATE_signed)
!33 = !DILocalVariable(name: "right", scope: !34, file: !18, line: 1463, type: !31, align: 8)
!34 = distinct !DILexicalBlock(scope: !30, file: !18, line: 1463)
!35 = !{}
!36 = !DILocation(line: 1461, scope: !17)
!37 = !DILocation(line: 1462, scope: !30)
!38 = !DILocation(line: 1463, scope: !34)
!39 = !DILocation(line: 1462, scope: !17)
!40 = !DILocalVariable(name: "self", arg: 1, scope: !41, file: !18, line: 1128, type: !24)
!41 = distinct !DILexicalBlock(scope: !42, file: !18, line: 1128)
!42 = distinct !DISubprogram(name: "to_bits", linkageName: "_ZN4core3f6421_$LT$impl$u20$f64$GT$7to_bits17he52a313848529cc1E", scope: !19, file: !18, line: 1128, type: !43, scopeLine: 1128, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !4, templateParams: !35, retainedNodes: !47)
!43 = !DISubroutineType(types: !44)
!44 = !{!45, !24}
!45 = !DIDerivedType(tag: DW_TAG_typedef, name: "u64", file: !8, baseType: !46)
!46 = !DIBasicType(name: "unsigned __int64", size: 64, encoding: DW_ATE_unsigned)
!47 = !{!40, !48}
!48 = !DILocalVariable(name: "self", arg: 1, scope: !49, file: !18, line: 1128, type: !24)
!49 = distinct !DILexicalBlock(scope: !42, file: !18, line: 1128)
!50 = !DILocation(line: 1128, scope: !41, inlinedAt: !39)
!51 = !DILocalVariable(name: "rt", arg: 1, scope: !52, file: !18, line: 1150, type: !24)
!52 = distinct !DILexicalBlock(scope: !53, file: !18, line: 1150)
!53 = distinct !DISubprogram(name: "rt_f64_to_u64", linkageName: "_ZN4core3f6421_$LT$impl$u20$f64$GT$7to_bits13rt_f64_to_u6417h44d856b0ff43abdeE", scope: !54, file: !18, line: 1150, type: !43, scopeLine: 1150, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !4, templateParams: !35, retainedNodes: !55)
!54 = !DINamespace(name: "to_bits", scope: !19)
!55 = !{!51, !56}
!56 = !DILocalVariable(name: "rt", arg: 1, scope: !57, file: !18, line: 1150, type: !24)
!57 = distinct !DILexicalBlock(scope: !53, file: !18, line: 1150)
!58 = !DILocation(line: 1150, scope: !52, inlinedAt: !59)
!59 = !DILocation(line: 1156, scope: !41, inlinedAt: !39)
!60 = !DILocation(line: 1154, scope: !52, inlinedAt: !59)
!61 = !DILocation(line: 1463, scope: !30)
!62 = !DILocation(line: 1128, scope: !49, inlinedAt: !61)
!63 = !DILocation(line: 1150, scope: !57, inlinedAt: !64)
!64 = !DILocation(line: 1156, scope: !49, inlinedAt: !61)
!65 = !DILocation(line: 1154, scope: !57, inlinedAt: !64)
!66 = !DILocation(line: 1487, scope: !34)
!67 = !DILocation(line: 1488, scope: !34)
!68 = !DILocation(line: 1490, scope: !34)
!69 = !DILocalVariable(name: "self", arg: 1, scope: !70, file: !71, line: 1575, type: !77)
!70 = distinct !DILexicalBlock(scope: !72, file: !71, line: 1575)
!71 = !DIFile(filename: "/rustc/eeb90cda1969383f56a2637cbd3037bdf598841c\\library\\core\\src\\cmp.rs", directory: "", checksumkind: CSK_SHA256, checksum: "2b7c057b9f23c850325c1389dbdedd7081a54c9ed236bbcb1c4cd036bc12647b")
!72 = distinct !DISubprogram(name: "cmp", linkageName: "_ZN4core3cmp5impls48_$LT$impl$u20$core..cmp..Ord$u20$for$u20$i64$GT$3cmp17h7805766c842055d4E", scope: !73, file: !71, line: 1575, type: !75, scopeLine: 1575, flags: DIFlagPrototyped, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !4, templateParams: !35, retainedNodes: !78)
!73 = !DINamespace(name: "impl$79", scope: !74)
!74 = !DINamespace(name: "impls", scope: !9)
!75 = !DISubroutineType(types: !76)
!76 = !{!7, !77, !77}
!77 = !DIDerivedType(tag: DW_TAG_pointer_type, name: "ref$<i64>", baseType: !31, size: 64, align: 64, dwarfAddressSpace: 0)
!78 = !{!69, !79}
!79 = !DILocalVariable(name: "other", arg: 2, scope: !70, file: !71, line: 1575, type: !77)
!80 = !DILocation(line: 1575, scope: !70, inlinedAt: !68)
!81 = !DILocation(line: 1576, scope: !70, inlinedAt: !68)
!82 = !DILocation(line: 1491, scope: !17)
!83 = distinct !DISubprogram(name: "probe", linkageName: "_ZN26autocfg_92ffc430b5b2e973_15probe17h9dc646ddc0469464E", scope: !85, file: !84, line: 1, type: !86, scopeLine: 1, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !4, templateParams: !35)
!84 = !DIFile(filename: "<anon>", directory: "", checksumkind: CSK_SHA256, checksum: "4413f23fc88f80784bc38c0596e470b2a62bd5c29edf8d750709442bb82c9abb")
!85 = !DINamespace(name: "autocfg_92ffc430b5b2e973_1", scope: null)
!86 = !DISubroutineType(types: !87)
!87 = !{null}
!88 = !DILocation(line: 1, scope: !83)
!89 = !DILocation(line: 1, scope: !90)
!90 = !DILexicalBlockFile(scope: !83, file: !84, discriminator: 0)
