%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define private void @Main_condFalse(ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %i_0_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_1, i8 0, i1 false)
   ret void
}

define private void @Main_condTrue(ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %i_0_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_1, i8 0, i1 false)
   ret void
}

define private void @Main_final(ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %i_0_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_1, i8 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_12_2 = alloca %struct.siko_Tuple_, align 4
   %i_12_1 = alloca %struct.siko_Tuple_, align 4
   %i_10_2 = alloca %struct.siko_Tuple_, align 4
   %i_10_1 = alloca %struct.siko_Tuple_, align 4
   %i_7_3 = alloca %struct.siko_Tuple_, align 4
   %i_7_2 = alloca %struct.siko_Tuple_, align 4
   %i_7_1 = alloca %struct.siko_Tuple_, align 4
   %i_6_2 = alloca %struct.siko_Tuple_, align 4
   %i_6_1 = alloca %struct.siko_Tuple_, align 4
   %i_4_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_2 = alloca %struct.Bool_Bool, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %i_0_1)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_0, i8 0, i1 false)
   call void @Bool_Bool_True(ptr %i_1_2)
   br label %block8
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_0_1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   %i_3_1 = bitcast %struct.Bool_Bool* %i_0_1 to %struct.siko_Tuple_*
   br label %block4
block4:
   call void @siko_Tuple_(ptr %i_4_1)
   br label %block1
block5:
   %i_5_1 = bitcast %struct.Bool_Bool* %i_0_1 to %struct.siko_Tuple_*
   br label %block6
block6:
   call void @Main_condTrue(ptr %i_6_1)
   call void @siko_Tuple_(ptr %i_6_2)
   br label %block1
block7:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_7_1, ptr align 4 %match_var_1, i8 0, i1 false)
   call void @Main_final(ptr %i_7_2)
   call void @siko_Tuple_(ptr %i_7_3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_7_3, i8 0, i1 false)
   ret void
block8:
   %tmp_switch_var_block8_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_1_2, i32 0, i32 0
   %tmp_switch_var_block8_2 = load i32, ptr %tmp_switch_var_block8_1, align 4
   switch i32 %tmp_switch_var_block8_2, label %block9 [
i32 1, label %block11
]

block9:
   %i_9_1 = bitcast %struct.Bool_Bool* %i_1_2 to %struct.siko_Tuple_*
   br label %block10
block10:
   call void @Main_condFalse(ptr %i_10_1)
   call void @siko_Tuple_(ptr %i_10_2)
   br label %block7
block11:
   %i_11_1 = bitcast %struct.Bool_Bool* %i_1_2 to %struct.siko_Tuple_*
   br label %block12
block12:
   call void @Main_condTrue(ptr %i_12_1)
   call void @siko_Tuple_(ptr %i_12_2)
   br label %block7
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


