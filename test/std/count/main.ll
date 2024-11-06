%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_8_4 = alloca %struct.Int_Int, align 8
   %i_8_2 = alloca %struct.Int_Int, align 8
   %i_8_1 = alloca %struct.Int_Int, align 8
   %i_6_1 = alloca %struct.Int_Int, align 8
   %i_2_2 = alloca %struct.siko_Tuple_, align 4
   %i_2_1 = alloca %struct.Int_Int, align 8
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_5 = alloca %struct.Bool_Bool, align 4
   %i_1_3 = alloca %struct.Int_Int, align 8
   %i_1_2 = alloca %struct.Int_Int, align 8
   %a_1 = alloca %struct.Int_Int, align 8
   %loop_var_0 = alloca %struct.Int_Int, align 8
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_0_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %i_0_1, i8 8, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_1, ptr align 8 %i_0_1, i8 8, i1 false)
   %tmp_i_1_2_1 = getelementptr inbounds %struct.Int_Int, ptr %i_1_2, i32 0, i32 0
   store i64 10, ptr %tmp_i_1_2_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_1_3, ptr align 8 %a_1, i8 8, i1 false)
   call void @Int_Int_lessThan(ptr %i_1_3, ptr %i_1_2, ptr %i_1_5)
   br label %block4
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_2_1, ptr align 8 %loop_var_0, i8 8, i1 false)
   call void @siko_Tuple_(ptr %i_2_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_2_2, i8 0, i1 false)
   ret void
block3:
block4:
   %tmp_switch_var_block4_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_1_5, i32 0, i32 0
   %tmp_switch_var_block4_2 = load i32, ptr %tmp_switch_var_block4_1, align 4
   switch i32 %tmp_switch_var_block4_2, label %block5 [
i32 1, label %block7
]

block5:
   %i_5_1 = bitcast %struct.Bool_Bool* %i_1_5 to %struct.siko_Tuple_*
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_6_1, ptr align 8 %a_1, i8 8, i1 false)
   br label %block2
block7:
   %i_7_1 = bitcast %struct.Bool_Bool* %i_1_5 to %struct.siko_Tuple_*
   br label %block8
block8:
   %tmp_i_8_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_8_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_8_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_8_2, ptr align 8 %a_1, i8 8, i1 false)
   call void @Int_Int_add(ptr %i_8_2, ptr %i_8_1, ptr %i_8_4)
   br label %block1
}

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


