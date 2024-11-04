%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool__Bool_Bool = type { %struct.Bool_Bool, %struct.Bool_Bool }

define void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @siko_Tuple_Bool_Bool__Bool_Bool(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool__Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f0, ptr align 4 %field0, i8 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %f1, ptr align 4 %field1, i8 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 8, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_15_1 = alloca %struct.Int_Int, align 8
   %i_13_1 = alloca %struct.Int_Int, align 8
   %i_9_1 = alloca %struct.Int_Int, align 8
   %i_7_1 = alloca %struct.Int_Int, align 8
   %i_1_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.Int_Int, align 8
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_3 = alloca %struct.siko_Tuple_Bool_Bool__Bool_Bool, align 4
   %i_0_2 = alloca %struct.Bool_Bool, align 4
   %i_0_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %i_0_1)
   call void @Bool_Bool_False(ptr %i_0_2)
   call void @siko_Tuple_Bool_Bool__Bool_Bool(ptr %i_0_3)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_1_1, ptr align 8 %match_var_0, i8 8, i1 false)
   call void @siko_Tuple_(ptr %i_1_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_2, i8 0, i1 false)
   ret void
block2:
   %i_2_1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %i_0_3, i32 0, i32 0
   %i_2_2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %i_0_3, i32 0, i32 1
   br label %block3
block3:
   %tmp_switch_var_block3_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_2_1, i32 0, i32 0
   %tmp_switch_var_block3_2 = load i32, ptr %tmp_switch_var_block3_1, align 4
   switch i32 %tmp_switch_var_block3_2, label %block4 [
i32 1, label %block10
]

block4:
   %i_4_1 = bitcast %struct.Bool_Bool* %i_2_1 to %struct.siko_Tuple_*
   br label %block5
block5:
   %tmp_switch_var_block5_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_2_2, i32 0, i32 0
   %tmp_switch_var_block5_2 = load i32, ptr %tmp_switch_var_block5_1, align 4
   switch i32 %tmp_switch_var_block5_2, label %block6 [
i32 1, label %block8
]

block6:
   %i_6_1 = bitcast %struct.Bool_Bool* %i_2_2 to %struct.siko_Tuple_*
   br label %block7
block7:
   %tmp_i_7_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_7_1, i32 0, i32 0
   store i64 4, ptr %tmp_i_7_1_1, align 8
   br label %block1
block8:
   %i_8_1 = bitcast %struct.Bool_Bool* %i_2_2 to %struct.siko_Tuple_*
   br label %block9
block9:
   %tmp_i_9_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_9_1, i32 0, i32 0
   store i64 4, ptr %tmp_i_9_1_1, align 8
   br label %block1
block10:
   %i_10_1 = bitcast %struct.Bool_Bool* %i_2_1 to %struct.siko_Tuple_*
   br label %block11
block11:
   %tmp_switch_var_block11_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_2_2, i32 0, i32 0
   %tmp_switch_var_block11_2 = load i32, ptr %tmp_switch_var_block11_1, align 4
   switch i32 %tmp_switch_var_block11_2, label %block12 [
i32 1, label %block14
]

block12:
   %i_12_1 = bitcast %struct.Bool_Bool* %i_2_2 to %struct.siko_Tuple_*
   br label %block13
block13:
   %tmp_i_13_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_13_1, i32 0, i32 0
   store i64 4, ptr %tmp_i_13_1_1, align 8
   br label %block1
block14:
   %i_14_1 = bitcast %struct.Bool_Bool* %i_2_2 to %struct.siko_Tuple_*
   br label %block15
block15:
   %tmp_i_15_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_15_1, i32 0, i32 0
   store i64 4, ptr %tmp_i_15_1_1, align 8
   br label %block1
}

define void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_False*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define void @Bool_Bool_True(ptr noundef %fn_result) {
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


