%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool__Bool_Bool = type { %struct.Bool_Bool, %struct.Bool_Bool }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool__Bool_Bool(ptr noundef %f0, ptr noundef %f1, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool__Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field1, ptr align 4 %f1, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %lit_17 = alloca %struct.Int_Int, align 8
   %lit_14 = alloca %struct.Int_Int, align 8
   %lit_10 = alloca %struct.Int_Int, align 8
   %lit_7 = alloca %struct.Int_Int, align 8
   %unit_24 = alloca %struct.siko_Tuple_, align 4
   %matchValue_23 = alloca %struct.Int_Int, align 8
   %match_var_4 = alloca %struct.Int_Int, align 8
   %tuple_3 = alloca %struct.siko_Tuple_Bool_Bool__Bool_Bool, align 4
   %call_2 = alloca %struct.Bool_Bool, align 4
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %call_1)
   call void @Bool_Bool_False(ptr %call_2)
   call void @siko_Tuple_Bool_Bool__Bool_Bool(ptr %call_1, ptr %call_2, ptr %tuple_3)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %matchValue_23, ptr align 8 %match_var_4, i64 8, i1 false)
   call void @siko_Tuple_(ptr %unit_24)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_24, i64 0, i1 false)
   ret void
block2:
   %tupleField_5 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %tuple_3, i32 0, i32 0
   %tupleField_6 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Bool_Bool, ptr %tuple_3, i32 0, i32 1
   br label %block3
block3:
   %tmp_switch_var_block3_1 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_5, i32 0, i32 0
   %tmp_switch_var_block3_2 = load i32, ptr %tmp_switch_var_block3_1, align 4
   switch i32 %tmp_switch_var_block3_2, label %block4 [
i32 1, label %block10
]

block4:
   br label %block5
block5:
   %tmp_switch_var_block5_3 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_6, i32 0, i32 0
   %tmp_switch_var_block5_4 = load i32, ptr %tmp_switch_var_block5_3, align 4
   switch i32 %tmp_switch_var_block5_4, label %block6 [
i32 1, label %block8
]

block6:
   br label %block7
block7:
   %tmp_lit_7_5 = getelementptr inbounds %struct.Int_Int, ptr %lit_7, i32 0, i32 0
   store i64 4, ptr %tmp_lit_7_5, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_4, ptr align 8 %lit_7, i64 8, i1 false)
   br label %block1
block8:
   br label %block9
block9:
   %tmp_lit_10_6 = getelementptr inbounds %struct.Int_Int, ptr %lit_10, i32 0, i32 0
   store i64 4, ptr %tmp_lit_10_6, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_4, ptr align 8 %lit_10, i64 8, i1 false)
   br label %block1
block10:
   br label %block11
block11:
   %tmp_switch_var_block11_7 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_6, i32 0, i32 0
   %tmp_switch_var_block11_8 = load i32, ptr %tmp_switch_var_block11_7, align 4
   switch i32 %tmp_switch_var_block11_8, label %block12 [
i32 1, label %block14
]

block12:
   br label %block13
block13:
   %tmp_lit_14_9 = getelementptr inbounds %struct.Int_Int, ptr %lit_14, i32 0, i32 0
   store i64 4, ptr %tmp_lit_14_9, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_4, ptr align 8 %lit_14, i64 8, i1 false)
   br label %block1
block14:
   br label %block15
block15:
   %tmp_lit_17_10 = getelementptr inbounds %struct.Int_Int, ptr %lit_17, i32 0, i32 0
   store i64 4, ptr %tmp_lit_17_10, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_4, ptr align 8 %lit_17, i64 8, i1 false)
   br label %block1
}

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

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


