%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Option_Option_Bool_Bool = type { i32, [4 x i8] }

%struct.Option_Option_None_Bool_Bool = type { i32, %struct.siko_Tuple_ }

%struct.Option_Option_Some_Bool_Bool = type { i32, %struct.siko_Tuple_Bool_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool = type { %struct.Bool_Bool }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b24i4 = alloca %struct.siko_Tuple_, align 4
   %b24i2 = alloca %struct.Bool_Bool, align 4
   %a_11 = alloca %struct.Bool_Bool, align 4
   %b22i3 = alloca %struct.siko_Tuple_, align 4
   %b22i1 = alloca %struct.Bool_Bool, align 4
   %b19i2 = alloca %struct.siko_Tuple_, align 4
   %b19i1 = alloca %struct.siko_Tuple_, align 4
   %b18i4 = alloca %struct.siko_Tuple_, align 4
   %b18i2 = alloca %struct.Bool_Bool, align 4
   %a_8 = alloca %struct.Bool_Bool, align 4
   %b16i3 = alloca %struct.siko_Tuple_, align 4
   %b16i1 = alloca %struct.Bool_Bool, align 4
   %match_var_10 = alloca %struct.siko_Tuple_, align 4
   %b13i4 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_9 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b13i2 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b13i1 = alloca %struct.siko_Tuple_, align 4
   %b12i2 = alloca %struct.Bool_Bool, align 4
   %a_5 = alloca %struct.Bool_Bool, align 4
   %b10i1 = alloca %struct.Bool_Bool, align 4
   %match_var_7 = alloca %struct.siko_Tuple_, align 4
   %b7i6 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_6 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b7i4 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b7i2 = alloca %struct.Bool_Bool, align 4
   %b7i1 = alloca %struct.Bool_Bool, align 4
   %b6i2 = alloca %struct.Bool_Bool, align 4
   %a_2 = alloca %struct.Bool_Bool, align 4
   %b4i1 = alloca %struct.Bool_Bool, align 4
   %match_var_4 = alloca %struct.Bool_Bool, align 4
   %b1i4 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_3 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b1i2 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b1i1 = alloca %struct.Bool_Bool, align 4
   %match_var_1 = alloca %struct.Bool_Bool, align 4
   %b0i5 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_0 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b0i3 = alloca %struct.Option_Option_Bool_Bool, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %b0i1)
   call void @Option_Option_Some_Bool_Bool(ptr %b0i1, ptr %b0i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %b0i3, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i5, ptr align 4 %a_0, i64 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_1, i64 4, i1 false)
   call void @Option_Option_None_Bool_Bool(ptr %b1i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_3, ptr align 4 %b1i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i4, ptr align 4 %a_3, i64 8, i1 false)
   br label %block8
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %b0i5, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Bool_Bool_False(ptr %b4i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b4i1, i64 4, i1 false)
   br label %block1
block5:
   %tmp_b5i1_1 = bitcast %struct.Option_Option_Bool_Bool* %b0i5 to %struct.Option_Option_Some_Bool_Bool*
   %b5i1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_b5i1_1, i32 0, i32 1
   %b5i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b5i1, i32 0, i32 0
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_2, ptr align 4 %b5i2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b6i2, ptr align 4 %a_2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b6i2, i64 4, i1 false)
   br label %block1
block7:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b7i1, ptr align 4 %match_var_4, i64 4, i1 false)
   call void @Bool_Bool_True(ptr %b7i2)
   call void @Option_Option_Some_Bool_Bool(ptr %b7i2, ptr %b7i4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_6, ptr align 4 %b7i4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b7i6, ptr align 4 %a_6, i64 8, i1 false)
   br label %block14
block8:
   %tmp_switch_var_block8_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %b1i4, i32 0, i32 0
   %tmp_switch_var_block8_2 = load i32, ptr %tmp_switch_var_block8_1, align 4
   switch i32 %tmp_switch_var_block8_2, label %block9 [
i32 1, label %block11
]

block9:
   br label %block10
block10:
   call void @Bool_Bool_False(ptr %b10i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_4, ptr align 4 %b10i1, i64 4, i1 false)
   br label %block7
block11:
   %tmp_b11i1_1 = bitcast %struct.Option_Option_Bool_Bool* %b1i4 to %struct.Option_Option_Some_Bool_Bool*
   %b11i1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_b11i1_1, i32 0, i32 1
   %b11i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b11i1, i32 0, i32 0
   br label %block12
block12:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_5, ptr align 4 %b11i2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b12i2, ptr align 4 %a_5, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_4, ptr align 4 %b12i2, i64 4, i1 false)
   br label %block7
block13:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b13i1, ptr align 4 %match_var_7, i64 0, i1 false)
   call void @Option_Option_None_Bool_Bool(ptr %b13i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_9, ptr align 4 %b13i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b13i4, ptr align 4 %a_9, i64 8, i1 false)
   br label %block20
block14:
   %tmp_switch_var_block14_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %b7i6, i32 0, i32 0
   %tmp_switch_var_block14_2 = load i32, ptr %tmp_switch_var_block14_1, align 4
   switch i32 %tmp_switch_var_block14_2, label %block15 [
i32 1, label %block17
]

block15:
   br label %block16
block16:
   call void @Bool_Bool_False(ptr %b16i1)
   call void @Std_Basic_Util_assert(ptr %b16i1, ptr %b16i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_7, ptr align 4 %b16i3, i64 0, i1 false)
   br label %block13
block17:
   %tmp_b17i1_1 = bitcast %struct.Option_Option_Bool_Bool* %b7i6 to %struct.Option_Option_Some_Bool_Bool*
   %b17i1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_b17i1_1, i32 0, i32 1
   %b17i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b17i1, i32 0, i32 0
   br label %block18
block18:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_8, ptr align 4 %b17i2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b18i2, ptr align 4 %a_8, i64 4, i1 false)
   call void @Std_Basic_Util_assert(ptr %b18i2, ptr %b18i4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_7, ptr align 4 %b18i4, i64 0, i1 false)
   br label %block13
block19:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b19i1, ptr align 4 %match_var_10, i64 0, i1 false)
   call void @siko_Tuple_(ptr %b19i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b19i2, i64 0, i1 false)
   ret void
block20:
   %tmp_switch_var_block20_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %b13i4, i32 0, i32 0
   %tmp_switch_var_block20_2 = load i32, ptr %tmp_switch_var_block20_1, align 4
   switch i32 %tmp_switch_var_block20_2, label %block21 [
i32 1, label %block23
]

block21:
   br label %block22
block22:
   call void @Bool_Bool_True(ptr %b22i1)
   call void @Std_Basic_Util_assert(ptr %b22i1, ptr %b22i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_10, ptr align 4 %b22i3, i64 0, i1 false)
   br label %block19
block23:
   %tmp_b23i1_1 = bitcast %struct.Option_Option_Bool_Bool* %b13i4 to %struct.Option_Option_Some_Bool_Bool*
   %b23i1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_b23i1_1, i32 0, i32 1
   %b23i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b23i1, i32 0, i32 0
   br label %block24
block24:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_11, ptr align 4 %b23i2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b24i2, ptr align 4 %a_11, i64 4, i1 false)
   call void @Std_Basic_Util_assert(ptr %b24i2, ptr %b24i4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_10, ptr align 4 %b24i4, i64 0, i1 false)
   br label %block19
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %b4i1)
   call void @siko_Tuple_(ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_False, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Option_Option_None_Bool_Bool(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Option_Option_None_Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Option_Option_None_Bool_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Option_Option_None_Bool_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define private void @Option_Option_Some_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Option_Option_Some_Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


